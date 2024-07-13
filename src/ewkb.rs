//! Module contenant les objets permettant d'encoder/décoder au format Extended Well-Known Bytes
//! (EWKB)
//!
//! Voir [self::EWKBGeometry]
use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Read, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    io::{Decodable, Encodable},
    types::{
        CoordinatesRef, GeometryImpl as _, LineString, LineStringZ, MultiLineString,
        MultiLineStringZ, MultiPoint, MultiPointZ, MultiPolygon, MultiPolygonZ, Point, PointZ,
        Polygon, PolygonZ, Vector, VectorArray, VectorMatrix, VectorTensor,
    },
};

use super::types::{Geometry, GeometryKind};

/// Objet intermédiaire pour encoder/decoder
/// au format EWKB toute géométrie.
#[derive(Debug, Clone, PartialEq)]
pub struct EWKBGeometry(pub(crate) Geometry);

impl Encodable for EWKBGeometry {
    fn encode<W: Write>(&self, stream: &mut W) -> Result<(), std::io::Error> {
        encode_geometry(&self.0, stream)
    }
}

impl Decodable for EWKBGeometry {
    fn decode<R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        decode_geometry(stream).map(Self)
    }
}

impl TryFrom<&[u8]> for EWKBGeometry {
    type Error = std::io::Error;

    fn try_from(mut value: &[u8]) -> Result<Self, Self::Error> {
        decode_geometry(&mut value).map(Self::new)
    }
}

impl From<EWKBGeometry> for Geometry {
    fn from(value: EWKBGeometry) -> Self {
        value.0
    }
}

impl From<Geometry> for EWKBGeometry {
    fn from(value: Geometry) -> Self {
        Self(value)
    }
}

impl Deref for EWKBGeometry {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EWKBGeometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EWKBGeometry {
    /// Crée une nouvelle géométrie encodable au format EWKB
    pub fn new<G: Into<Geometry>>(args: G) -> Self {
        Self(args.into())
    }
}

impl_geometry_proxies!(EWKB);

const BIG_ENDIAN: u8 = 0;
const LITTLE_ENDIAN: u8 = 1;

/// Objet permettant d'encoder ou décoder suivant le boutisme définit dans la base de données.
pub enum Endianess {
    BigEndian,
    LittleEndian,
}

impl From<PhantomData<BigEndian>> for Endianess {
    fn from(_value: PhantomData<BigEndian>) -> Self {
        Endianess::BigEndian
    }
}

impl From<PhantomData<LittleEndian>> for Endianess {
    fn from(_value: PhantomData<LittleEndian>) -> Self {
        Endianess::LittleEndian
    }
}

impl From<Endianess> for u8 {
    fn from(value: Endianess) -> Self {
        match value {
            Endianess::BigEndian => BIG_ENDIAN,
            Endianess::LittleEndian => LITTLE_ENDIAN,
        }
    }
}

pub fn encode_geometry<W: Write>(
    geometry: &Geometry,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    encode_geometry_with_endianess::<NativeEndian, _>(geometry, stream)
}

pub fn encode_geometry_with_endianess<E: ByteOrder, W: Write>(
    geom: &Geometry,
    stream: &mut W,
) -> Result<(), std::io::Error>
where
    Endianess: From<PhantomData<E>>,
{
    // Write endianess.
    stream.write_u8(Endianess::from(PhantomData::<E>).into())?;

    // Write the EWKB flags
    let flags = Flags::from_geometry(geom);
    encode_flags::<E, _>(&flags, stream)?;

    // Write the SRID, if any
    if let Some(srid) = geom.srid() {
        stream.write_u32::<E>(srid)?;
    }

    // Encode the coordinate
    encode_coordinates::<E, _>(geom.borrow_coordinates(), stream)
}

pub fn decode_geometry<R: Read>(stream: &mut R) -> Result<Geometry, std::io::Error> {
    // start byte must be 0x00
    assert_eq!(stream.read_u8()?, 0);

    let endian = stream.read_u8()?;

    if endian == BIG_ENDIAN {
        decode_geometry_with_endianess::<BigEndian, _>(stream)
    } else if endian == LITTLE_ENDIAN {
        decode_geometry_with_endianess::<LittleEndian, _>(stream)
    } else {
        panic!("wrong value for endianess")
    }
}

fn decode_geometry_with_endianess<E: ByteOrder, R: Read>(
    stream: &mut R,
) -> Result<Geometry, std::io::Error> {
    let flags = decode_flags::<E, _>(stream)?;
    let srid: Option<u32> = if flags.with_srid {
        Some(stream.read_u32::<E>()?)
    } else {
        None
    };

    let mut geom: Geometry = match flags.kind {
        GeometryKind::Point => Point::new(decode_vector::<2, E, _>(stream)?).into(),
        GeometryKind::LineString => LineString::new(decode_array::<2, E, _>(stream)?).into(),
        GeometryKind::Polygon => Polygon::new(decode_matrix::<2, E, _>(stream)?).into(),
        GeometryKind::MultiPoint => MultiPoint::new(decode_array::<2, E, _>(stream)?).into(),
        GeometryKind::MultiLineString => {
            MultiLineString::new(decode_matrix::<2, E, _>(stream)?).into()
        }
        GeometryKind::MultiPolygon => MultiPolygon::new(decode_tensor::<2, E, _>(stream)?).into(),
        GeometryKind::GeometryCollection => todo!(),
        GeometryKind::PointZ => PointZ::new(decode_vector::<3, E, _>(stream)?).into(),
        GeometryKind::LineStringZ => LineStringZ::new(decode_array::<3, E, _>(stream)?).into(),
        GeometryKind::PolygonZ => PolygonZ::new(decode_matrix::<3, E, _>(stream)?).into(),
        GeometryKind::MultiPointZ => MultiPointZ::new(decode_array::<3, E, _>(stream)?).into(),
        GeometryKind::MultiLineStringZ => {
            MultiLineStringZ::new(decode_matrix::<3, E, _>(stream)?).into()
        }
        GeometryKind::MultiPolygonZ => MultiPolygonZ::new(decode_tensor::<3, E, _>(stream)?).into(),
        GeometryKind::GeometryCollectionZ => todo!(),
    };

    geom.set_srid(srid);

    Ok(geom)
}

/// The flags for the EWKB codec.
/// Source: [https://github.com/postgis/postgis/blob/master/doc/ZMSgeoms.txt]
struct Flags {
    kind: GeometryKind,
    with_srid: bool,
}

impl Flags {
    pub fn from_geometry(geom: &Geometry) -> Self {
        Self {
            kind: geom.kind(),
            with_srid: geom.srid().is_some(),
        }
    }
}

const WITH_SRID_MASK: u32 = 0x20000000;

fn decode_flags<E: ByteOrder, R: Read>(stream: &mut R) -> Result<Flags, std::io::Error> {
    let encoded = stream.read_u32::<E>()?;

    let with_srid = (encoded & WITH_SRID_MASK) == WITH_SRID_MASK;

    let kind = match encoded & !WITH_SRID_MASK {
        1 => GeometryKind::Point,
        2 => GeometryKind::LineString,
        3 => GeometryKind::Polygon,
        4 => GeometryKind::MultiPoint,
        5 => GeometryKind::MultiLineString,
        6 => GeometryKind::MultiPolygon,
        7 => GeometryKind::GeometryCollection,

        0x80000001 => GeometryKind::PointZ,
        0x80000002 => GeometryKind::LineStringZ,
        0x80000003 => GeometryKind::PolygonZ,
        0x80000004 => GeometryKind::MultiPointZ,
        0x80000005 => GeometryKind::MultiLineStringZ,
        0x80000006 => GeometryKind::MultiPolygonZ,
        0x80000007 => GeometryKind::GeometryCollectionZ,

        _ => panic!("unhandled geometry class"),
    };

    Ok(Flags { kind, with_srid })
}

fn encode_flags<E: ByteOrder, W: Write>(
    flags: &Flags,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    let mut encoded = match flags.kind {
        GeometryKind::Point => 1,
        GeometryKind::LineString => 2,
        GeometryKind::Polygon => 3,
        GeometryKind::MultiPoint => 4,
        GeometryKind::MultiLineString => 5,
        GeometryKind::MultiPolygon => 6,
        GeometryKind::GeometryCollection => 7,

        GeometryKind::PointZ => 0x80000001,
        GeometryKind::LineStringZ => 0x80000002,
        GeometryKind::PolygonZ => 0x80000003,
        GeometryKind::MultiPointZ => 0x80000004,
        GeometryKind::MultiLineStringZ => 0x80000005,
        GeometryKind::MultiPolygonZ => 0x80000006,
        GeometryKind::GeometryCollectionZ => 0x80000007,
    };

    encoded |= if flags.with_srid { WITH_SRID_MASK } else { 0 };

    stream.write_u32::<E>(encoded)
}

fn encode_coordinates<E: ByteOrder, W: Write>(
    coordinates: CoordinatesRef<'_>,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    match coordinates {
        CoordinatesRef::Vector2D(vector) => encode_vector::<2, E, _>(vector, stream),
        CoordinatesRef::VectorArray2D(array) => encode_array::<2, E, _>(array, stream),
        CoordinatesRef::VectorMatrix2D(matrix) => encode_matrix::<2, E, _>(matrix, stream),
        CoordinatesRef::VectorTensor2D(tensor) => encode_tensor::<2, E, _>(tensor, stream),
        CoordinatesRef::Vector3D(vector) => encode_vector::<3, E, _>(vector, stream),
        CoordinatesRef::VectorArray3D(array) => encode_array::<3, E, _>(array, stream),
        CoordinatesRef::VectorMatrix3D(matrix) => encode_matrix::<3, E, _>(matrix, stream),
        CoordinatesRef::VectorTensor3D(tensor) => encode_tensor::<3, E, _>(tensor, stream),
    }
}

fn encode_vector<const N: usize, E: ByteOrder, W: Write>(
    vector: &Vector<N, f64>,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    vector
        .iter()
        .copied()
        .try_for_each(|scalar| stream.write_f64::<E>(scalar))
}

fn decode_vector<const N: usize, E: ByteOrder, R: Read>(
    stream: &mut R,
) -> Result<Vector<N, f64>, std::io::Error> {
    let mut scalars: [f64; N] = [0f64; N];

    for i in 0..N {
        scalars[i] = stream.read_f64::<E>()?;
    }

    Ok(Vector::new(scalars))
}

fn encode_array<const N: usize, E: ByteOrder, W: Write>(
    array: &VectorArray<N, f64>,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    stream.write_u32::<E>(array.len() as u32)?;
    array
        .iter()
        .try_for_each(|vector| encode_vector::<N, E, _>(vector, stream))
}

fn decode_array<const N: usize, E: ByteOrder, R: Read>(
    stream: &mut R,
) -> Result<VectorArray<N, f64>, std::io::Error> {
    let nb_points: u32 = stream.read_u32::<E>()?;
    let mut coordinates = Vec::<Vector<N, f64>>::with_capacity(nb_points as usize);

    for _ in 0..nb_points {
        coordinates.push(decode_vector::<N, E, _>(stream)?);
    }

    Ok(VectorArray::new(coordinates))
}

fn encode_matrix<const N: usize, E: ByteOrder, W: Write>(
    matrix: &VectorMatrix<N, f64>,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    stream.write_u32::<E>(matrix.len() as u32)?;
    matrix
        .iter()
        .try_for_each(|array| encode_array::<N, E, _>(array, stream))
}

fn decode_matrix<const N: usize, E: ByteOrder, R: Read>(
    stream: &mut R,
) -> Result<VectorMatrix<N, f64>, std::io::Error> {
    let nb_points: u32 = stream.read_u32::<E>()?;
    let mut coordinates = Vec::<VectorArray<N, f64>>::with_capacity(nb_points as usize);

    for _ in 0..nb_points {
        coordinates.push(decode_array::<N, E, _>(stream)?);
    }

    Ok(VectorMatrix::new(coordinates))
}

fn encode_tensor<const N: usize, E: ByteOrder, W: Write>(
    tensor: &VectorTensor<N, f64>,
    stream: &mut W,
) -> Result<(), std::io::Error> {
    stream.write_u32::<E>(tensor.len() as u32)?;
    tensor
        .iter()
        .try_for_each(|matrix| encode_matrix::<N, E, _>(matrix, stream))
}

fn decode_tensor<const N: usize, E: ByteOrder, R: Read>(
    stream: &mut R,
) -> Result<VectorTensor<N, f64>, std::io::Error> {
    let nb_points: u32 = stream.read_u32::<E>()?;
    let mut coordinates = Vec::<VectorMatrix<N, f64>>::with_capacity(nb_points as usize);

    for _ in 0..nb_points {
        coordinates.push(decode_matrix::<N, E, _>(stream)?);
    }

    Ok(VectorTensor::new(coordinates))
}

#[cfg(test)]
mod tests {
    use crate::types::GeometryImpl;

    use super::*;

    #[test]
    pub fn test_isomorphism_ewkb() {
        let expected = EWKBGeometry::new(Point::new([10.0, 20.0]));
        let bytes = expected.encode_to_vec().expect("cannot encode geometry");
        let value = EWKBGeometry::decode_from_slice(&bytes).expect("cannot decode from stream");
        assert_eq!(value, expected)
    }
}
