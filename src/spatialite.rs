use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};

use std::{
    io::{Read, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{io::{Decodable, Encodable}, types::{
    CoordinatesRef, Geometry, GeometryImpl as _, GeometryKind, LineString, LineStringZ, MultiLineString, MultiLineStringZ, MultiPoint, MultiPointZ, MultiPolygon, MultiPolygonZ, Point, PointZ, Polygon, PolygonZ, Vector, VectorArray, VectorMatrix, VectorTensor, MBR
}, DEFAULT_SRID};

const BIG_ENDIAN: u8 = 0;
const LITTLE_ENDIAN: u8 = 1;

#[derive(Debug, Clone, PartialEq)]
/// Objet intermédiaire pour encoder/décoder une géométrie au format natif de SpatiaLite.
pub struct SpatiaLiteGeometry(Geometry);

impl Encodable for SpatiaLiteGeometry {
    fn encode<W: Write>(&self, stream: &mut W) -> Result<(), std::io::Error> {
        encode_geometry(&self.0, stream)
    }
}

impl Decodable for SpatiaLiteGeometry {
    fn decode<R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        decode_geometry(stream).map(Self::new)
    }
}

impl SpatiaLiteGeometry {
    pub fn new<G: Into<Geometry>>(args: G) -> Self {
        Self(args.into())
    }

    pub fn into_geometry(self) -> Geometry {
        self.0
    }
}

impl Deref for SpatiaLiteGeometry {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpatiaLiteGeometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Geometry> for SpatiaLiteGeometry {
    fn from(value: Geometry) -> Self {
        Self(value)
    }
}

impl From<SpatiaLiteGeometry> for Geometry {
    fn from(value: SpatiaLiteGeometry) -> Self {
        value.0
    }
}

impl_geometry_proxies!(SpatiaLite);

/// Implémente l'encodage / décodage depuis sqlx
mod sqlx {
    use super::*;
    use ::sqlx::{sqlite::SqliteValueRef, Database, Decode, Encode, Sqlite, Type};

    impl<'r, DB> Type<DB> for SpatiaLiteGeometry
    where
        DB: Database,
        &'r [u8]: Type<DB>,
    {
        fn type_info() -> <DB as Database>::TypeInfo {
            <&[u8]>::type_info()
        }
    }

    impl<'r> Decode<'r, Sqlite> for SpatiaLiteGeometry
    {
        fn decode(
            value: SqliteValueRef<'r>,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let mut encoded = <&'r [u8] as Decode<'r, Sqlite>>::decode(value)?;
            let geom = decode_geometry(&mut encoded).map(Self::new)?;
            Ok(geom)
        }
    }

    impl<'q, DB> Encode<'q, DB> for SpatiaLiteGeometry
    where
        DB: Database,
        Vec<u8>: Encode<'q, DB>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <DB as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            let encoded = self.encode_to_vec().unwrap();
            encoded.encode_by_ref(buf)
        }
    }

    impl_geometry_sqlx_codecs!(SpatiaLite);
}


pub fn encode_geometry<W: Write>(geometry: &Geometry, stream: &mut W) -> Result<(), std::io::Error> {
    encode_geometry_with_endianess::<NativeEndian, _>(geometry, stream)
}

pub fn encode_geometry_with_endianess<E: ByteOrder, W: Write>(geometry: &Geometry, stream: &mut W) -> Result<(), std::io::Error> 
where Endianess: From<PhantomData<E>>
{
    // encode start byte, always 0x00
    stream.write_u8(0)?;
    
    // encode endianness 
    stream.write_u8(Endianess::from(PhantomData::<E>).into())?;
   
    // encode SRID
    stream.write_u32::<E>(geometry.srid().unwrap_or(DEFAULT_SRID))?;
    
    // encode MBR
    encode_mbr::<E, _>(&geometry.mbr(), stream)?;
    
    // encode geometry class
    encode_geometry_class::<E, _>(&geometry.kind(), stream)?;

    // encode the coordinates
    encode_coordinates::<E, _>(geometry.borrow_coordinates(), stream)?;

    // a GEOMETRY encoded BLOB value must always end with a 0xFE byte
    stream.write_u8(0xFE)
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

fn decode_geometry_with_endianess<E: ByteOrder, R: Read>(stream: &mut R) -> Result<Geometry, std::io::Error> {
    // Read the SRID
    let srid: u32 = stream.read_u32::<E>()?;

    // Read MBR
    let _mbr = decode_mbr::<E, _>(stream)?;

    // Read the geometry class
    let kind = decode_geometry_class::<E, _>(stream)?;

    // Decode the coordinates depending of the geometry class.
    let mut geom: Geometry = match kind {
        GeometryKind::Point => Point::new(decode_vector::<2, E, _>(stream)?).into(),
        GeometryKind::LineString => LineString::new(decode_array::<2, E, _>(stream)?).into(),
        GeometryKind::Polygon => Polygon::new(decode_matrix::<2, E, _>(stream)?).into(),
        GeometryKind::MultiPoint => MultiPoint::new(decode_array::<2, E, _>(stream)?).into(),
        GeometryKind::MultiLineString => MultiLineString::new(decode_matrix::<2, E, _>(stream)?).into(),
        GeometryKind::MultiPolygon => MultiPolygon::new(decode_tensor::<2, E, _>(stream)?).into(),
        GeometryKind::GeometryCollection => todo!(),
        GeometryKind::PointZ => PointZ::new(decode_vector::<3, E, _>(stream)?).into(),
        GeometryKind::LineStringZ => LineStringZ::new(decode_array::<3, E, _>(stream)?).into(),
        GeometryKind::PolygonZ => PolygonZ::new(decode_matrix::<3, E, _>(stream)?).into(),
        GeometryKind::MultiPointZ => MultiPointZ::new(decode_array::<3, E, _>(stream)?).into(),
        GeometryKind::MultiLineStringZ => MultiLineStringZ::new(decode_matrix::<3, E, _>(stream)?).into(),
        GeometryKind::MultiPolygonZ => MultiPolygonZ::new(decode_tensor::<3, E, _>(stream)?).into(),
        GeometryKind::GeometryCollectionZ => todo!(),
    };

    geom.set_srid(Some(srid));
    
    let end = stream.read_u8()?;
    assert_eq!(end, 0xFE);

    Ok(geom)
}

fn encode_geometry_class<E: ByteOrder, W: Write>(kind: &GeometryKind, stream: &mut W) -> Result<(), std::io::Error> {
    let encoded = match kind {
        GeometryKind::Point => 1,
        GeometryKind::LineString => 2,
        GeometryKind::Polygon => 3,
        GeometryKind::MultiPoint => 4,
        GeometryKind::MultiLineString => 5,
        GeometryKind::MultiPolygon => 6,
        GeometryKind::GeometryCollection => 7,
        GeometryKind::PointZ => 1001,
        GeometryKind::LineStringZ => 1002,
        GeometryKind::PolygonZ => 1003,
        GeometryKind::MultiPointZ => 1004,
        GeometryKind::MultiLineStringZ => 10015,
        GeometryKind::MultiPolygonZ => 1006,
        GeometryKind::GeometryCollectionZ => 1007,
    };

    stream.write_u32::<E>(encoded)
}

fn decode_geometry_class<E: ByteOrder, R: Read>(stream: &mut R) -> Result<GeometryKind, std::io::Error> {
    Ok(match stream.read_u32::<E>()? {
        1 => GeometryKind::Point,
        2 => GeometryKind::LineString,
        3 => GeometryKind::Polygon,
        4 => GeometryKind::MultiPoint,
        5 => GeometryKind::MultiLineString,
        6 => GeometryKind::MultiPolygon,
        7 => GeometryKind::GeometryCollection,

        1001 => GeometryKind::PointZ,
        1002 => GeometryKind::LineStringZ,
        1003 => GeometryKind::PolygonZ,
        1004 => GeometryKind::MultiPointZ,
        1005 => GeometryKind::MultiLineStringZ,
        1006 => GeometryKind::MultiPolygonZ,
        1007 => GeometryKind::MultiLineStringZ,

        _ => panic!("unknown WKB geometry"),
    })
}

fn encode_mbr<E: ByteOrder, W: Write>(mbr: &MBR<f64>, stream: &mut W) -> Result<(), std::io::Error> {
    stream.write_f64::<E>(mbr.min_x)?;
    stream.write_f64::<E>(mbr.min_y)?;
    stream.write_f64::<E>(mbr.max_x)?;
    stream.write_f64::<E>(mbr.max_y)?;
    stream.write_u8(0x7C)
}

fn decode_mbr<E: ByteOrder, R: Read>(stream: &mut R) -> Result<MBR<f64>, std::io::Error> {
    let min_x = stream.read_f64::<E>()?;
    let min_y = stream.read_f64::<E>()?;
    let max_x = stream.read_f64::<E>()?;
    let max_y = stream.read_f64::<E>()?;
    let mbr_end = stream.read_u8()?;

    assert_eq!(mbr_end, 0x7C);

    Ok(MBR {
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

fn encode_coordinates<E: ByteOrder, W: Write>(coordinates: CoordinatesRef<'_>, stream: &mut W) -> Result<(), std::io::Error> {
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

fn encode_vector<const N: usize, E: ByteOrder, W: Write>(vector: &Vector<N, f64>, stream: &mut W) -> Result<(), std::io::Error> {
    vector.iter().copied().map(|scalar| stream.write_f64::<E>(scalar)).collect::<Result<_, _>>()
}

fn decode_vector<const N: usize, E: ByteOrder, R: Read>(stream: &mut R) -> Result<Vector<N, f64>, std::io::Error> {
    let mut scalars: [f64; N] = [0f64; N];

    for i in 0..N {
        scalars[i] = stream.read_f64::<E>()?;
    }

    Ok(Vector::new(scalars))
}


fn encode_array<const N: usize, E: ByteOrder, W: Write>(array: &VectorArray<N, f64>, stream: &mut W) -> Result<(), std::io::Error> {
        stream.write_u32::<E>(array.len() as u32)?;
        array.iter().map(|vector| encode_vector::<N, E, _>(vector, stream)).collect::<Result<_, _>>()
}

fn decode_array<const N: usize, E: ByteOrder, R: Read>(stream: &mut R) -> Result<VectorArray<N, f64>, std::io::Error> {
    let nb_points: u32 = stream.read_u32::<E>()?;
    let mut coordinates = Vec::<Vector<N, f64>>::with_capacity(nb_points as usize);

    for _ in 0..nb_points {
        coordinates.push(decode_vector::<N, E, _>(stream)?);
    }

    Ok(VectorArray::new(coordinates))
}

fn encode_matrix<const N: usize, E: ByteOrder, W: Write>(matrix: &VectorMatrix<N, f64>, stream: &mut W) -> Result<(), std::io::Error> {
        stream.write_u32::<E>(matrix.len() as u32)?;
        matrix.iter().map(|array| encode_array::<N, E, _>(array, stream)).collect::<Result<_, _>>()
}

fn decode_matrix<const N: usize, E: ByteOrder, R: Read>(stream: &mut R) -> Result<VectorMatrix<N, f64>, std::io::Error> {
    let nb_points: u32 = stream.read_u32::<E>()?;
    let mut coordinates = Vec::<VectorArray<N, f64>>::with_capacity(nb_points as usize);

    for _ in 0..nb_points {
        coordinates.push(decode_array::<N, E, _>(stream)?);
    }

    Ok(VectorMatrix::new(coordinates))
}

fn encode_tensor<const N: usize, E: ByteOrder, W: Write>(tensor: &VectorTensor<N, f64>, stream: &mut W) -> Result<(), std::io::Error> {
    stream.write_u32::<E>(tensor.len() as u32)?;
    tensor.iter().map(|matrix| encode_matrix::<N, E, _>(matrix, stream)).collect::<Result<_, _>>()
}

fn decode_tensor<const N: usize, E: ByteOrder, R: Read>(stream: &mut R) -> Result<VectorTensor<N, f64>, std::io::Error> {
    let nb_points: u32 = stream.read_u32::<E>()?;
    let mut coordinates = Vec::<VectorMatrix<N, f64>>::with_capacity(nb_points as usize);

    for _ in 0..nb_points {
        coordinates.push(decode_matrix::<N, E, _>(stream)?);
    }

    Ok(VectorTensor::new(coordinates))
}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_isomorphism() {
        let expected = SpatiaLiteGeometry::new(Point::new([10.0, 20.0]));
        let bytes = expected.encode_to_vec().expect("cannot encode geometry");
        let value = SpatiaLiteGeometry::decode_from_slice(&bytes).expect("cannot decode geometry");
        assert_eq!(value, expected)
    }
}
