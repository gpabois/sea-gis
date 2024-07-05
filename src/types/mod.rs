use line_string::LineStringCoordinates;
use multi_line_string::MultiLineStringCoordinates;
use multi_point::MultiPointCoordinates;
use multi_polygon::MultiPolygonCoordinates;
use point::PointCoordinates;
use polygon::PolygonCoordinates;
use serde::{de::{Visitor, Error as _}, ser::{SerializeMap, Error as _}, Deserialize, Serialize};

mod vectors;
mod mbr;
mod point;
mod multi_point;
mod line_string;
mod multi_line_string;
mod polygon;
mod multi_polygon;

pub use vectors::{Vector, VectorArray, VectorMatrix, VectorTensor};
pub use mbr::MBR;

pub type Vector2D = Vector<2, f64>;
pub type VectorArray2D = VectorArray<2, f64>;
pub type VectorMatrix2D = VectorMatrix<2, f64>;
pub type VectorTensor2D = VectorTensor<2, f64>;

pub type Vector3D = Vector<3, f64>;
pub type VectorArray3D = VectorArray<3, f64>;
pub type VectorMatrix3D = VectorMatrix<3, f64>;
pub type VectorTensor3D = VectorTensor<3, f64>;

/// Generic type for points, used internally keep it DRY.
pub(crate) type GenPoint<const N: usize, U> = point::Point<N, U>;
pub(crate) type GenLineString<const N: usize, U> = line_string::LineString<N, U>;
pub(crate) type GenPolygon<const N: usize, U> = polygon::Polygon<N, U>;
pub(crate) type GenMultiPoint<const N: usize, U> = multi_point::MultiPoint<N, U>;
pub(crate) type GenMultiLineString<const N: usize, U> = multi_line_string::MultiLineString<N, U>;
pub(crate) type GenMultiPolygon<const N: usize, U> = multi_polygon::MultiPolygon<N, U>;

// A point in a 2D space.
pub type Point = point::Point<2, f64>;
pub type MultiPoint = multi_point::MultiPoint<2, f64>;
pub type LineString = line_string::LineString<2, f64>;
pub type MultiLineString = multi_line_string::MultiLineString<2, f64>;
pub type Polygon = polygon::Polygon<2, f64>;
pub type MultiPolygon = multi_polygon::MultiPolygon<2, f64>;

// A point in a 3D space.
pub type PointZ = point::Point<3, f64>;
pub type MultiPointZ = multi_point::MultiPoint<3, f64>;
pub type LineStringZ = line_string::LineString<3, f64>;
pub type MultiLineStringZ = multi_line_string::MultiLineString<3, f64>;
pub type PolygonZ = polygon::Polygon<3, f64>;
pub type MultiPolygonZ = multi_polygon::MultiPolygon<3, f64>;


/// Représente toutes les géométries possibles.
#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),

    PointZ(PointZ),
    LineStringZ(LineStringZ),
    PolygonZ(PolygonZ),
    MultiPointZ(MultiPointZ),
    MultiLineStringZ(MultiLineStringZ),
    MultiPolygonZ(MultiPolygonZ),
}

impl Geometry {
    /// Emprunte les coordonnées d'une géométrie.
    pub fn borrow_coordinates(&self) -> CoordinatesRef<'_> {
        match self {
            Geometry::Point(a) => CoordinatesRef::Vector2D(&a.coordinates),
            Geometry::LineString(a) => CoordinatesRef::VectorArray2D(&a.coordinates),
            Geometry::Polygon(a) => CoordinatesRef::VectorMatrix2D(&a.coordinates),
            Geometry::MultiPoint(a) => CoordinatesRef::VectorArray2D(&a.coordinates),
            Geometry::MultiLineString(a) => CoordinatesRef::VectorMatrix2D(&a.coordinates),
            Geometry::MultiPolygon(a) => CoordinatesRef::VectorTensor2D(&a.coordinates),
            Geometry::PointZ(a) => CoordinatesRef::Vector3D(&a.coordinates),
            Geometry::LineStringZ(a) => CoordinatesRef::VectorArray3D(&a.coordinates),
            Geometry::PolygonZ(a) => CoordinatesRef::VectorMatrix3D(&a.coordinates),
            Geometry::MultiPointZ(a) => CoordinatesRef::VectorArray3D(&a.coordinates),
            Geometry::MultiLineStringZ(a) => CoordinatesRef::VectorMatrix3D(&a.coordinates),
            Geometry::MultiPolygonZ(a) => CoordinatesRef::VectorTensor3D(&a.coordinates),
        }
    }

    pub fn kind(&self) -> GeometryKind {
        match self {
            Geometry::Point(_) => GeometryKind::Point,
            Geometry::LineString(_) => GeometryKind::LineString,
            Geometry::Polygon(_) => GeometryKind::Polygon,
            Geometry::MultiPoint(_) => GeometryKind::MultiPolygon,
            Geometry::MultiLineString(_) => GeometryKind::MultiLineString,
            Geometry::MultiPolygon(_) => GeometryKind::MultiPolygon,
            Geometry::PointZ(_) => GeometryKind::PointZ,
            Geometry::LineStringZ(_) => GeometryKind::LineStringZ,
            Geometry::PolygonZ(_) => GeometryKind::PolygonZ,
            Geometry::MultiPointZ(_) => GeometryKind::MultiPointZ,
            Geometry::MultiLineStringZ(_) => GeometryKind::MultiLineStringZ,
            Geometry::MultiPolygonZ(_) => GeometryKind::MultiPolygonZ,
        }
    }

    pub fn mbr(&self) -> MBR<f64> {
        match self {
            Geometry::Point(a) => a.mbr(),
            Geometry::LineString(a) => a.mbr(),
            Geometry::Polygon(a) => a.mbr(),
            Geometry::MultiPoint(a) => a.mbr(),
            Geometry::MultiLineString(a) => a.mbr(),
            Geometry::MultiPolygon(a) => a.mbr(),
            Geometry::PointZ(a) => a.mbr(),
            Geometry::LineStringZ(a) => a.mbr(),
            Geometry::PolygonZ(a) => a.mbr(),
            Geometry::MultiPointZ(a) => a.mbr(),
            Geometry::MultiLineStringZ(a) => a.mbr(),
            Geometry::MultiPolygonZ(a) => a.mbr(),
        }
    }

    pub fn set_srid(&mut self, srid: u32) {
        match self {
            Geometry::Point(a) => a.srid = srid,
            Geometry::LineString(a) => a.srid = srid,
            Geometry::Polygon(a) => a.srid = srid,
            Geometry::MultiPoint(a) => a.srid = srid,
            Geometry::MultiLineString(a) => a.srid = srid,
            Geometry::MultiPolygon(a) => a.srid = srid,
            Geometry::PointZ(a) => a.srid = srid,
            Geometry::LineStringZ(a) => a.srid = srid,
            Geometry::PolygonZ(a) => a.srid = srid,
            Geometry::MultiPointZ(a) => a.srid = srid,
            Geometry::MultiLineStringZ(a) => a.srid = srid,
            Geometry::MultiPolygonZ(a) => a.srid = srid,
        }
    }

    pub fn srid(&self) -> u32 {
        match self {
            Geometry::Point(p) => p.srid,
            Geometry::LineString(ls) => ls.srid,
            Geometry::Polygon(p) => p.srid,
            Geometry::MultiPoint(a) => a.srid,
            Geometry::MultiLineString(a) => a.srid,
            Geometry::MultiPolygon(a) => a.srid,
            Geometry::PointZ(a) => a.srid,
            Geometry::LineStringZ(a) => a.srid,
            Geometry::PolygonZ(a) => a.srid,
            Geometry::MultiPointZ(a) => a.srid,
            Geometry::MultiLineStringZ(a) => a.srid,
            Geometry::MultiPolygonZ(a) => a.srid,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Représente la classe de la géométrie.
pub enum GeometryKind {
    /// 2D point
    Point,
    /// 2D line string
    LineString,
    /// 2D polygon
    Polygon,
    /// 2D set of Point
    MultiPoint,
    /// 2D set of line strings
    MultiLineString,
    /// 2D set of Polygon
    MultiPolygon,
    /// 2D set of geometries
    GeometryCollection,

    /// 3D point
    PointZ,
    /// 3D line string
    LineStringZ,
    /// 3D polygon
    PolygonZ,
    /// 3D set of Point
    MultiPointZ,
    /// 3D set of line strings
    MultiLineStringZ,
    /// 3D set of Polygon
    MultiPolygonZ,
    /// 3D set of geometries
    GeometryCollectionZ,
}

pub const POINT_KIND_STR: &str = "Point";
pub const LINE_STRING_KIND_STR: &str = "LineString";
pub const POLYGON_KIND_STR: &str = "Polygon";
pub const MULTI_POINT_KIND_STR: &str = "MultiPoint";
pub const MULTI_LINE_STRING_KIND_STR: &str = "MultiLineString";
pub const MULTI_POLYGON_KIND_STR: &str = "MultiPolygon";
pub const GEOMETRY_COLLECTION_KIND_STR: &str = "GeometryCollection";

impl AsRef<str> for GeometryKind {
    fn as_ref(&self) -> &str {
        match self {
            GeometryKind::Point => POINT_KIND_STR,
            GeometryKind::LineString => LINE_STRING_KIND_STR,
            GeometryKind::Polygon => POLYGON_KIND_STR,
            GeometryKind::MultiPoint => MULTI_POINT_KIND_STR,
            GeometryKind::MultiLineString => MULTI_LINE_STRING_KIND_STR,
            GeometryKind::MultiPolygon => MULTI_POLYGON_KIND_STR,
            GeometryKind::GeometryCollection => GEOMETRY_COLLECTION_KIND_STR,
            GeometryKind::PointZ => POINT_KIND_STR,
            GeometryKind::LineStringZ => LINE_STRING_KIND_STR,
            GeometryKind::PolygonZ => POLYGON_KIND_STR,
            GeometryKind::MultiPointZ => MULTI_POINT_KIND_STR,
            GeometryKind::MultiLineStringZ => MULTI_LINE_STRING_KIND_STR,
            GeometryKind::MultiPolygonZ => MULTI_POLYGON_KIND_STR,
            GeometryKind::GeometryCollectionZ => GEOMETRY_COLLECTION_KIND_STR,
        }
    }
}
impl std::fmt::Display for GeometryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = self.as_ref();
        write!(f, "{kind_str}")
    }
}


impl From<Point> for Geometry {
    fn from(value: Point) -> Self {
        Self::Point(value)
    }
}

impl TryFrom<Geometry> for Point {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::Point(point) => Ok(point),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::Point,
                value.kind(),
            )),
        }
    }
}

impl From<LineString> for Geometry {
    fn from(value: LineString) -> Self {
        Self::LineString(value)
    }
}

impl TryFrom<Geometry> for LineString {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::LineString(line_string) => Ok(line_string),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::LineString,
                value.kind(),
            )),
        }
    }
}

impl From<Polygon> for Geometry {
    fn from(value: Polygon) -> Self {
        Self::Polygon(value)
    }
}

impl TryFrom<Geometry> for Polygon {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::Polygon(polygon) => Ok(polygon),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::Polygon,
                value.kind(),
            )),
        }
    }
}

impl From<MultiPoint> for Geometry {
    fn from(value: MultiPoint) -> Self {
        Self::MultiPoint(value)
    }
}

impl TryFrom<Geometry> for MultiPoint {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::MultiPoint(a) => Ok(a),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::MultiPoint,
                value.kind(),
            )),
        }
    }
}

impl From<MultiLineString> for Geometry {
    fn from(value: MultiLineString) -> Self {
        Self::MultiLineString(value)
    }
}

impl TryFrom<Geometry> for MultiLineString {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::MultiLineString(line_strings) => Ok(line_strings),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::MultiLineString,
                value.kind(),
            )),
        }
    }
}

impl From<MultiPolygon> for Geometry {
    fn from(value: MultiPolygon) -> Self {
        Self::MultiPolygon(value)
    }
}

impl TryFrom<Geometry> for MultiPolygon {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::MultiPolygon(a) => Ok(a),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::MultiPolygon,
                value.kind(),
            )),
        }
    }
}

impl From<PointZ> for Geometry {
    fn from(value: PointZ) -> Self {
        Self::PointZ(value)
    }
}

impl TryFrom<Geometry> for PointZ {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::PointZ(point) => Ok(point),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::PointZ,
                value.kind(),
            )),
        }
    }
}

impl From<LineStringZ> for Geometry {
    fn from(value: LineStringZ) -> Self {
        Self::LineStringZ(value)
    }
}

impl TryFrom<Geometry> for LineStringZ {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::LineStringZ(line_string) => Ok(line_string),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::LineStringZ,
                value.kind(),
            )),
        }
    }
}

impl From<PolygonZ> for Geometry {
    fn from(value: PolygonZ) -> Self {
        Self::PolygonZ(value)
    }
}

impl TryFrom<Geometry> for PolygonZ {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::PolygonZ(polygon) => Ok(polygon),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::PolygonZ,
                value.kind(),
            )),
        }
    }
}

impl From<MultiPointZ> for Geometry {
    fn from(value: MultiPointZ) -> Self {
        Self::MultiPointZ(value)
    }
}

impl TryFrom<Geometry> for MultiPointZ {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::MultiPointZ(a) => Ok(a),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::MultiPointZ,
                value.kind(),
            )),
        }
    }
}

impl From<MultiLineStringZ> for Geometry {
    fn from(value: MultiLineStringZ) -> Self {
        Self::MultiLineStringZ(value)
    }
}

impl TryFrom<Geometry> for MultiLineStringZ {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::MultiLineStringZ(line_strings) => Ok(line_strings),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::MultiLineStringZ,
                value.kind(),
            )),
        }
    }
}

impl From<MultiPolygonZ> for Geometry {
    fn from(value: MultiPolygonZ) -> Self {
        Self::MultiPolygonZ(value)
    }
}

impl TryFrom<Geometry> for MultiPolygonZ {
    type Error = super::error::Error;

    fn try_from(value: Geometry) -> Result<Self, Self::Error> {
        match value {
            Geometry::MultiPolygonZ(a) => Ok(a),
            _ => Err(super::error::Error::invalid_geometry_kind(
                GeometryKind::MultiPolygonZ,
                value.kind(),
            )),
        }
    }
}

/// Coordonnées d'une géométrie.
pub enum Coordinates {
    Vector2D(Vector<2, f64>),
    VectorArray2D(VectorArray<2, f64>),
    VectorMatrix2D(VectorMatrix<2, f64>),
    VectorTensor2D(VectorTensor<2, f64>),

    Vector3D(Vector<3, f64>),
    VectorArray3D( VectorArray<3, f64>),
    VectorMatrix3D(VectorMatrix<3, f64>),
    VectorTensor3D(VectorTensor<3, f64>)
}

/// Coordonnées empruntées d'une géométrie.
pub enum CoordinatesMutRef<'a> {
    Vector2D(&'a mut Vector<2, f64>),
    VectorArray2D(&'a mut VectorArray<2, f64>),
    VectorMatrix2D(&'a mut VectorMatrix<2, f64>),
    VectorTensor2D(&'a mut VectorTensor<2, f64>),

    Vector3D(&'a mut Vector<3, f64>),
    VectorArray3D(&'a mut VectorArray<3, f64>),
    VectorMatrix3D(&'a mut VectorMatrix<3, f64>),
    VectorTensor3D(&'a mut VectorTensor<3, f64>)
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut Vector<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::Vector2D(a) => Ok(a),
            _ => panic!("not a 2D vector")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut VectorArray<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::VectorArray2D(a) => Ok(a),
            _ => panic!("not an array of 2D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut VectorMatrix<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::VectorMatrix2D(a) => Ok(a),
            _ => panic!("not a matrix of 2D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut VectorTensor<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::VectorTensor2D(a) => Ok(a),
            _ => panic!("not a matrix of 2D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut Vector<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::Vector3D(a) => Ok(a),
            _ => panic!("not a 3D vector")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut VectorArray<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::VectorArray3D(a) => Ok(a),
            _ => panic!("not an array of 3D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut VectorMatrix<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::VectorMatrix3D(a) => Ok(a),
            _ => panic!("not a matrix of 3D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesMutRef<'a>> for &'a mut VectorTensor<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesMutRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesMutRef::VectorTensor3D(a) => Ok(a),
            _ => panic!("not a matrix of 3D vectors")
        }
    }
}

/// Coordonnées empruntées d'une géométrie.
pub enum CoordinatesRef<'a> {
    Vector2D(&'a Vector<2, f64>),
    VectorArray2D(&'a VectorArray<2, f64>),
    VectorMatrix2D(&'a VectorMatrix<2, f64>),
    VectorTensor2D(&'a VectorTensor<2, f64>),

    Vector3D(&'a Vector<3, f64>),
    VectorArray3D(&'a VectorArray<3, f64>),
    VectorMatrix3D(&'a VectorMatrix<3, f64>),
    VectorTensor3D(&'a VectorTensor<3, f64>)
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a Vector<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::Vector2D(a) => Ok(a),
            _ => panic!("not a 2D vector")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a VectorArray<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::VectorArray2D(a) => Ok(a),
            _ => panic!("not an array of 2D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a VectorMatrix<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::VectorMatrix2D(a) => Ok(a),
            _ => panic!("not a matrix of 2D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a VectorTensor<2, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::VectorTensor2D(a) => Ok(a),
            _ => panic!("not a matrix of 2D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a Vector<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::Vector3D(a) => Ok(a),
            _ => panic!("not a 3D vector")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a VectorArray<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::VectorArray3D(a) => Ok(a),
            _ => panic!("not an array of 3D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a VectorMatrix<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::VectorMatrix3D(a) => Ok(a),
            _ => panic!("not a matrix of 3D vectors")
        }
    }
}

impl<'a> TryFrom<CoordinatesRef<'a>> for &'a VectorTensor<3, f64> {
    type Error = crate::error::Error;

    fn try_from(value: CoordinatesRef<'a>) -> Result<Self, Self::Error> {
        match value {
            CoordinatesRef::VectorTensor3D(a) => Ok(a),
            _ => panic!("not a matrix of 3D vectors")
        }
    }
}
