use std::{io::{Cursor, Read, Write}, ops::{Deref, DerefMut}};
use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};

use crate::types::{self, LineString, LineStringS, LineStringZ, MultiLineString, MultiLineStringS, MultiLineStringZ, MultiPoint, MultiPointS, MultiPointZ, MultiPolygon, MultiPolygonS, MultiPolygonZ, PointS, PointZ, Polygon, PolygonS, PolygonZ, Vector, VectorArray, VectorMatrix, VectorTensor, MBR};

use super::types::{Geometry, GeometryKind, Point};

const BIG_ENDIAN: u8 = 0;
const LITTLE_ENDIAN: u8 = 1;

#[derive(Clone, PartialEq)]
/// Objet intermédiaire pour encoder/décoder une géométrie au format natif de SpatiaLite.
pub struct SpatiaLiteGeometry(Geometry);

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

/// Implémente l'encodage / décodage pour Sea ORM
mod sea_orm {
    use super::*;

    use sea_query::{Value, ValueType, ArrayType, ColumnType, Nullable};

    impl From<SpatiaLiteGeometry> for Value {
        fn from(value: SpatiaLiteGeometry) -> Self {
            let mut buf = Vec::<u8>::default();
            value.encode_to_stream(&mut buf).expect("cannot encode SpatiaLite geometry");
            buf.into()
        }
    }

    impl Nullable for SpatiaLiteGeometry {
        fn null() -> sea_orm::Value {
            sea_orm::Value::Bytes(None)
        }
    }
    
    impl ValueType for SpatiaLiteGeometry {
        fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
            match v {
                Value::Bytes(Some(boxed_buf)) => {
                    let mut buf = Cursor::new(boxed_buf.as_ref());
                    SpatiaLiteGeometry::decode_from_stream(&mut buf).map_err(|_| sea_query::ValueTypeErr)
                }
                _ => Err(sea_query::ValueTypeErr),
            }
        }
    
        fn type_name() -> String {
            stringify!(EWKBGeometry).to_owned()
        }
    
        fn array_type() -> sea_query::ArrayType {
            ArrayType::Bytes
        }
    
        fn column_type() -> sea_orm::ColumnType {
            ColumnType::Bit(None)
        }
    }
    
}

/// Implémente l'encodage / décodage depuis sqlx
mod sqlx {
    use sqlx::{Database, Decode, Encode};
    use super::SpatiaLiteGeometry;


    impl<'r, DB> Decode<'r,DB> for SpatiaLiteGeometry 
    where DB: Database, &'r [u8]: Decode<'r, DB>
    {
        fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, sqlx::error::BoxDynError> {
            let encoded = <&'r [u8] as Decode<DB>>::decode(value)?;
            let decoded = Self::try_from(encoded)?;
            Ok(decoded)
        }
    } 

    impl<'q, DB> Encode<'q, DB> for SpatiaLiteGeometry 
    where DB: Database, Vec<u8>: Encode<'q, DB>
    {
        fn encode_by_ref(&self, buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
            let mut encoded = Vec::<u8>::new();
            self.clone().encode_to_stream(&mut encoded).expect("cannot encode to SpatiaLite internal format");
            encoded.encode_by_ref(buf)
        }
        
        fn encode(self, buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull
        where
            Self: Sized,
        {
            let mut encoded = Vec::<u8>::new();
            self.encode_to_stream(&mut encoded).expect("cannot encode to SpatiaLite internal format");
            encoded.encode(buf)
        }
    }
}

impl TryFrom<&[u8]> for SpatiaLiteGeometry {
    type Error = std::io::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(value);
        Self::decode_from_stream(&mut cursor)
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

impl SpatiaLiteGeometry 
{
    pub fn encode_to_stream<W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.encode_to_stream_with_endianess::<NativeEndian, _>(stream)
    }

    pub fn encode_to_stream_with_endianess<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> 
    where Endianess: From<E>
    {
        stream.write_u8(0)?;
        stream.write_u32::<E>(self.srid())?;     
        self.mbr().encode_spatialite::<E, _>(stream)?;   
        self.kind().encode_spatialite::<E,_>(stream)?;

        match self.0 {
            Geometry::PointS(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::LineStringS(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::PolygonS(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::MultiPointS(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::MultiLineStringS(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::MultiPolygonS(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::PointZ(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::LineStringZ(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::PolygonZ(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::MultiPointZ(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::MultiLineStringZ(a) => a.encode_spatialite::<E,_>(stream)?,
            Geometry::MultiPolygonZ(a) => a.encode_spatialite::<E,_>(stream)?,
        }

        // a GEOMETRY encoded BLOB value must always end with a 0xFE byte
        stream.write_u8(0xFE)?;

        Ok(())
    }

    pub fn decode_from_stream<R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        let start = stream.read_u8()?;
        assert_eq!(start, 0u8);

        let endian = stream.read_u8()?;

        if endian == BIG_ENDIAN {
            Self::decode_from_stream_with_endianess::<BigEndian, _>(stream)
        } else {
            Self::decode_from_stream_with_endianess::<LittleEndian, _>(stream)
        }
    }

    pub fn decode_from_stream_with_endianess<E: ByteOrder, R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        // Read the SRID
        let srid: u32 = stream.read_u32::<E>()?;
        
        // Read MBR
        let _mbr = MBR::decode_spatialite::<E, _>(stream)?;
        
        // Read the geometry class
        let kind = GeometryKind::decode_spatialite::<E, _>(stream)?;

        let mut geometry: Geometry = match kind {
            GeometryKind::PointS => PointS::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::LineStringS => LineStringS::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::PolygonS => PolygonS::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiPointS => MultiPointS::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiLineStringS => MultiLineStringS::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiPolygonS => MultiPolygonS::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollectionS => todo!(),
            GeometryKind::PointZ => PointZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::LineStringZ => LineStringZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::PolygonZ => PolygonZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiPointZ => MultiPointZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiLineStringZ => MultiLineStringZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiPolygonZ => MultiPolygonZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollectionZ => todo!(),
        };

        geometry.set_srid(srid);

        Ok(Self(geometry))
    }
}

impl GeometryKind {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        let encoded = match self {
            GeometryKind::PointS => 1,
            GeometryKind::LineStringS => 2,
            GeometryKind::PolygonS => 3,
            GeometryKind::MultiPointS => 4,
            GeometryKind::MultiLineStringS => 5,
            GeometryKind::MultiPolygonS => 6,
            GeometryKind::GeometryCollectionS => 7,
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
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        Ok(match stream.read_u32::<E>()? {
            1 => GeometryKind::PointS,
            2 => GeometryKind::LineStringS,
            3 => GeometryKind::PolygonS,
            4 => GeometryKind::MultiPointS,
            5 => GeometryKind::MultiLineStringS,
            6 => GeometryKind::MultiPolygonS,
            7 => GeometryKind::GeometryCollectionS,

            1001 => GeometryKind::PointZ,
            1002 => GeometryKind::LineStringZ,
            1003 => GeometryKind::PolygonZ,
            1004 => GeometryKind::MultiPointZ,
            1005 => GeometryKind::MultiLineStringZ,
            1006 => GeometryKind::MultiPolygonZ,
            1007 => GeometryKind::MultiLineStringZ,

            _ => panic!("unknown WKB geometry")
        }.into())
    }
}

impl MBR<f64> {
    pub fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        stream.write_f64::<E>(self.min_x)?;
        stream.write_f64::<E>(self.min_y)?;
        stream.write_f64::<E>(self.max_x)?;
        stream.write_f64::<E>(self.max_y)?;
        stream.write_u8(0x7c)?;

        Ok(())
    }
    pub fn decode_spatialite<E: ByteOrder, R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        let min_x = stream.read_f64::<E>()?;
        let min_y = stream.read_f64::<E>()?;
        let max_x = stream.read_f64::<E>()?;
        let max_y = stream.read_f64::<E>()?;
        let mbr_end = stream.read_u8()?;      

        assert_eq!(mbr_end, 0x7c);

        Ok(MBR {min_x, max_x, min_y, max_y})
    }
}

impl<const N: usize> Point<N, f64> {
    /// Encode un point dans un flux binaire.
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = Vector::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> MultiPoint<N, f64> {
    /// Encode un ensemble de points dans un flux binaire
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorArray::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> LineString<N, f64> {
    /// Encode un ensemble de points dans un flux binaire
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorArray::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> MultiLineString<N, f64> {

    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> Polygon<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> MultiPolygon<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorTensor::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> Vector<N, f64> {
    /// Encode un vecteur N-D
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        for i in 0..N {
            stream.write_f64::<E>(self[i])?;
        }

        Ok(())
    }

    /// Décode un vecteur n-D
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let mut coordinates: [f64; N] = [0f64; N];

        for i in 0..N {
            coordinates[i] = value.read_f64::<E>()?;
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorArray<N, f64> {
    /// Encode une liste de vecteurs dans le flux de sortie.
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
            vector.encode_spatialite::<E, _>(stream)?;
        }

        Ok(())
    }

    /// Décode une liste de vecteurs depuis un flux d'entrée.
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(stream: &mut R) -> Result<Self, std::io::Error> {
        let nb_points: u32 = stream.read_u32::<E>()?;
        let mut coordinates = Vec::<Vector<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(Vector::decode_spatialite::<E, _>(stream)?);
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorMatrix<N, f64> {

    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
            vector.encode_spatialite::<E, _>(stream)?;
        }

        Ok(())
    }


    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let nb_points: u32 = value.read_u32::<E>()?;
        let mut coordinates = Vec::<VectorArray<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(VectorArray::decode_spatialite::<E, _>(value)?);
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorTensor<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
            vector.encode_spatialite::<E, _>(stream)?;
        }

        Ok(())
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let nb_points: u32 = value.read_u32::<E>()?;
        let mut coordinates = Vec::<VectorMatrix<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(VectorMatrix::decode_spatialite::<E, _>(value)?);
        }

        Ok(Self::new(coordinates))
    }
}

pub enum Endianess {
    BigEndian,
    LittleEndian,
}

impl From<BigEndian> for Endianess {
    fn from(_value: BigEndian) -> Self {
        Endianess::BigEndian
    }
}

impl From<LittleEndian> for Endianess {
    fn from(_value: LittleEndian) -> Self {
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
