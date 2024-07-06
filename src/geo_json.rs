use serde::{
    de::{self, Error as _, Visitor},
    ser::{SerializeMap as _, SerializeSeq},
    Deserialize, Serialize,
};
use std::ops::Deref;

use crate::types::{
    self, Coordinates, Geometry, GeometryKind, LineString, LineStringZ, MultiLineString,
    MultiLineStringZ, MultiPoint, MultiPointZ, MultiPolygon, MultiPolygonZ, Point, PointZ, Polygon,
    PolygonZ, Vector, VectorArray, VectorMatrix, VectorTensor, GEOMETRY_COLLECTION_KIND_STR,
    LINE_STRING_KIND_STR, MULTI_LINE_STRING_KIND_STR, MULTI_POINT_KIND_STR, MULTI_POLYGON_KIND_STR,
    POINT_KIND_STR, POLYGON_KIND_STR,
};

#[derive(Debug, PartialEq, Clone)]
/// Objet intermédiaire permettant d'encoder/décoder au format GeoJSON
pub struct GeoJsonGeometry(types::Geometry);

impl GeoJsonGeometry {
    pub fn new<G: Into<Geometry>>(args: G) -> Self {
        Self(args.into())
    }
}

impl Deref for GeoJsonGeometry {
    type Target = types::Geometry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for GeoJsonGeometry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;

        map.serialize_entry("type", self.kind().as_ref())?;
        map.serialize_entry(
            "coordinates",
            &GeoJsonCoordinatesRef(self.borrow_coordinates()),
        )?;

        map.end()
    }
}

impl<'de> Deserialize<'de> for GeoJsonGeometry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Self(deserializer.deserialize_map(GeometryVisitor {})?))
    }
}

struct GeometryVisitor {}

impl<'de> Visitor<'de> for GeometryVisitor {
    type Value = Geometry;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct GeoJsonGeometry")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut kind: Option<GeometryKind> = None;
        let mut coords: Option<Coordinates> = None;

        while let Some(key) = map.next_key()? {
            match key {
                "type" => {
                    kind = Some(map.next_value::<GeoJsonGeometryKind>()?.0);
                }
                "coordinates" => {
                    coords = Some(map.next_value::<GeoJsonCoordinates>()?.into());
                }
                _ => {}
            }
        }

        let kind = kind.ok_or_else(|| de::Error::missing_field("type"))?;
        let coords = coords.ok_or_else(|| de::Error::missing_field("coordinates"))?;

        let geom: Geometry = match (kind, coords) {
            (GeometryKind::Point, Coordinates::Vector2D(a)) => Point::new(a).into(),
            (GeometryKind::LineString, Coordinates::VectorArray2D(a)) => LineString::new(a).into(),
            (GeometryKind::Polygon, Coordinates::VectorMatrix2D(a)) => Polygon::new(a).into(),
            (GeometryKind::MultiPoint, Coordinates::VectorArray2D(a)) => MultiPoint::new(a).into(),
            (GeometryKind::MultiLineString, Coordinates::VectorMatrix2D(a)) => {
                MultiLineString::new(a).into()
            }
            (GeometryKind::MultiPolygon, Coordinates::VectorTensor2D(a)) => {
                MultiPolygon::new(a).into()
            }
            (GeometryKind::PointZ, Coordinates::Vector3D(a)) => PointZ::new(a).into(),
            (GeometryKind::LineStringZ, Coordinates::VectorArray3D(a)) => {
                LineStringZ::new(a).into()
            }
            (GeometryKind::PolygonZ, Coordinates::VectorMatrix3D(a)) => PolygonZ::new(a).into(),
            (GeometryKind::MultiPointZ, Coordinates::VectorArray3D(a)) => {
                MultiPointZ::new(a).into()
            }
            (GeometryKind::MultiLineStringZ, Coordinates::VectorMatrix3D(a)) => {
                MultiLineStringZ::new(a).into()
            }
            (GeometryKind::MultiPolygonZ, Coordinates::VectorTensor3D(a)) => {
                MultiPolygonZ::new(a).into()
            }
            _ => {
                return Err(A::Error::custom(format!(
                    "incompatible geometry type with the given coordinates"
                )))
            }
        };

        Ok(geom)
    }
}

struct GeoJsonGeometryKind(GeometryKind);

impl<'de> Deserialize<'de> for GeoJsonGeometryKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(deserializer.deserialize_str(GeometryKindVisitor {})?))
    }
}

struct GeometryKindVisitor {}

impl<'de> Visitor<'de> for GeometryKindVisitor {
    type Value = GeometryKind;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Point, LineString, Polgygon, MultiPoint, MultiLineString, MultiPolygon, or GeometryCollection")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            POINT_KIND_STR => Ok(GeometryKind::Point),
            LINE_STRING_KIND_STR => Ok(GeometryKind::LineString),
            POLYGON_KIND_STR => Ok(GeometryKind::Polygon),
            MULTI_POINT_KIND_STR => Ok(GeometryKind::MultiPoint),
            MULTI_LINE_STRING_KIND_STR => Ok(GeometryKind::MultiLineString),
            MULTI_POLYGON_KIND_STR => Ok(GeometryKind::MultiPolygon),
            GEOMETRY_COLLECTION_KIND_STR => Ok(GeometryKind::GeometryCollection),
            _ => Err(E::custom("expecting Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, or GeometryCollection"))
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GeoJsonCoordinates {
    Vector2D([f64; 2]),
    VectorArray2D(Vec<[f64; 2]>),
    VectorMatrix2D(Vec<Vec<[f64; 2]>>),
    VectorTensor2D(Vec<Vec<Vec<[f64; 2]>>>),

    Vector3D([f64; 3]),
    VectorArray3D(Vec<[f64; 3]>),
    VectorMatrix3D(Vec<Vec<[f64; 3]>>),
    VectorTensor3D(Vec<Vec<Vec<[f64; 3]>>>),
}

impl From<GeoJsonCoordinates> for Coordinates {
    fn from(value: GeoJsonCoordinates) -> Self {
        match value {
            GeoJsonCoordinates::Vector2D(a) => Coordinates::Vector2D(Vector::from(a)),
            GeoJsonCoordinates::VectorArray2D(a) => {
                Coordinates::VectorArray2D(VectorArray::from_iter(a))
            }
            GeoJsonCoordinates::VectorMatrix2D(a) => {
                Coordinates::VectorMatrix2D(VectorMatrix::from_iter(a))
            }
            GeoJsonCoordinates::VectorTensor2D(a) => {
                Coordinates::VectorTensor2D(VectorTensor::from_iter(a))
            }
            GeoJsonCoordinates::Vector3D(a) => Coordinates::Vector3D(Vector::from(a)),
            GeoJsonCoordinates::VectorArray3D(a) => {
                Coordinates::VectorArray3D(VectorArray::from_iter(a))
            }
            GeoJsonCoordinates::VectorMatrix3D(a) => {
                Coordinates::VectorMatrix3D(VectorMatrix::from_iter(a))
            }
            GeoJsonCoordinates::VectorTensor3D(a) => {
                Coordinates::VectorTensor3D(VectorTensor::from_iter(a))
            }
        }
    }
}

/// Référence à une coordonnée géométrique qui peut être encodé au format GeoJSON
struct GeoJsonCoordinatesRef<'a>(types::CoordinatesRef<'a>);

struct VectorRef<'a, const N: usize>(&'a Vector<N, f64>);

impl<const N: usize> Serialize for VectorRef<'_, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.deref().serialize(serializer)
    }
}

struct VectorArrayRef<'a, const N: usize>(&'a [Vector<N, f64>]);

impl<const N: usize> Serialize for VectorArrayRef<'_, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        self.0
            .iter()
            .map(|value| seq.serialize_element(&VectorRef(value)))
            .collect::<Result<_, _>>()?;
        seq.end()
    }
}

struct VectorMatrixRef<'a, const N: usize>(&'a [VectorArray<N, f64>]);

impl<const N: usize> Serialize for VectorMatrixRef<'_, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        self.0
            .iter()
            .map(|value| seq.serialize_element(&VectorArrayRef(value)))
            .collect::<Result<_, _>>()?;
        seq.end()
    }
}

struct VectorTensorRef<'a, const N: usize>(&'a [VectorMatrix<N, f64>]);

impl<const N: usize> Serialize for VectorTensorRef<'_, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        self.0
            .iter()
            .map(|value| seq.serialize_element(&VectorMatrixRef(value)))
            .collect::<Result<_, _>>()?;
        seq.end()
    }
}

impl Serialize for GeoJsonCoordinatesRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            types::CoordinatesRef::Vector2D(a) => VectorRef(a).serialize(serializer),
            types::CoordinatesRef::VectorArray2D(a) => VectorArrayRef(a).serialize(serializer),
            types::CoordinatesRef::VectorMatrix2D(a) => VectorMatrixRef(a).serialize(serializer),
            types::CoordinatesRef::VectorTensor2D(a) => VectorTensorRef(a).serialize(serializer),
            types::CoordinatesRef::Vector3D(a) => VectorRef(a).serialize(serializer),
            types::CoordinatesRef::VectorArray3D(a) => VectorArrayRef(a).serialize(serializer),
            types::CoordinatesRef::VectorMatrix3D(a) => VectorMatrixRef(a).serialize(serializer),
            types::CoordinatesRef::VectorTensor3D(a) => VectorTensorRef(a).serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};

    use super::GeoJsonGeometry;

    #[test]
    fn test_isomorphism_geo_json_point() {
        let expected = GeoJsonGeometry::new(Point::new([10.0, 20.0]));

        let encoded = serde_json::to_string(&expected).expect("cannot serialize to GeoJSON");
        print!("{encoded}");

        let value = serde_json::from_str::<GeoJsonGeometry>(&encoded)
            .expect("cannot deserialize from GeoJSON");

        assert_eq!(value, expected)
    }

    #[test]
    fn test_isomorphism_geo_json_line_string() {
        let expected = GeoJsonGeometry::new(LineString::new([[10.0, 20.0], [15.0, 25.0]]));

        let encoded = serde_json::to_string(&expected).expect("cannot serialize to GeoJSON");
        print!("{encoded}");

        let value = serde_json::from_str::<GeoJsonGeometry>(&encoded)
            .expect("cannot deserialize from GeoJSON");

        assert_eq!(value, expected)
    }

    #[test]
    fn test_isomorphism_geo_json_polygon() {
        let expected = GeoJsonGeometry::new(Polygon::new([[10.0, 20.0], [15.0, 25.0]]));

        let encoded = serde_json::to_string(&expected).expect("cannot serialize to GeoJSON");
        print!("{encoded}");

        let value = serde_json::from_str::<GeoJsonGeometry>(&encoded)
            .expect("cannot deserialize from GeoJSON");

        assert_eq!(value, expected)
    }

    #[test]
    fn test_isomorphism_geo_json_multi_point() {
        let expected = GeoJsonGeometry::new(MultiPoint::new([[10.0, 20.0], [15.0, 25.0]]));

        assert_eq!(expected.kind().as_ref(), "MultiPoint");

        let encoded = serde_json::to_string(&expected).expect("cannot serialize to GeoJSON");
        print!("{encoded}");

        let value = serde_json::from_str::<GeoJsonGeometry>(&encoded)
            .expect("cannot deserialize from GeoJSON");

        assert_eq!(value, expected)
    }

    #[test]
    fn test_isomorphism_geo_json_multi_line_string() {
        let expected = GeoJsonGeometry::new(MultiLineString::new([[10.0, 20.0], [15.0, 25.0]]));

        let encoded = serde_json::to_string(&expected).expect("cannot serialize to GeoJSON");
        print!("{encoded}");

        let value = serde_json::from_str::<GeoJsonGeometry>(&encoded)
            .expect("cannot deserialize from GeoJSON");

        assert_eq!(value, expected)
    }

    #[test]
    fn test_isomorphism_geo_json_multi_polygon() {
        let expected = GeoJsonGeometry::new(MultiPolygon::new([[10.0, 20.0], [15.0, 25.0]]));

        let encoded = serde_json::to_string(&expected).expect("cannot serialize to GeoJSON");
        print!("{encoded}");

        let value = serde_json::from_str::<GeoJsonGeometry>(&encoded)
            .expect("cannot deserialize from GeoJSON");

        assert_eq!(value, expected)
    }
}
