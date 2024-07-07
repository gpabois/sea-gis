use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};

use std::{
    io::{Cursor, Read, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::types::{
    GenLineString, GenMultiLineString, GenMultiPoint, GenMultiPolygon, GenPoint, GenPolygon,
    Geometry, GeometryKind, LineString, LineStringZ, MultiLineString, MultiLineStringZ, MultiPoint,
    MultiPointZ, MultiPolygon, MultiPolygonZ, Point, PointZ, Polygon, PolygonZ, Vector,
    VectorArray, VectorMatrix, VectorTensor, MBR,
};

const BIG_ENDIAN: u8 = 0;
const LITTLE_ENDIAN: u8 = 1;

#[derive(Debug, Clone, PartialEq)]
/// Objet intermédiaire pour encoder/décoder une géométrie au format natif de SpatiaLite.
pub struct SpatiaLiteGeometry(Geometry);

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

/// Implémente l'encodage / décodage pour Sea ORM
mod sea_orm {
    use super::*;

    use sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType};

    impl From<SpatiaLiteGeometry> for Value {
        fn from(value: SpatiaLiteGeometry) -> Self {
            let mut buf = Vec::<u8>::default();
            value
                .encode_to_stream(&mut buf)
                .expect("cannot encode SpatiaLite geometry");
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
                    SpatiaLiteGeometry::decode_from_stream(&mut buf)
                        .map_err(|_| sea_query::ValueTypeErr)
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
    use super::*;
    use ::sqlx::{Database, Decode, Encode, Type};

    impl<'r, DB> Type<DB> for SpatiaLiteGeometry
    where
        DB: Database,
        &'r [u8]: Type<DB>,
    {
        fn type_info() -> <DB as Database>::TypeInfo {
            <&[u8]>::type_info()
        }
    }

    impl<'r, DB> Decode<'r, DB> for SpatiaLiteGeometry
    where
        DB: Database,
        &'r [u8]: Decode<'r, DB>,
    {
        fn decode(
            value: <DB as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let encoded = <&'r [u8] as Decode<DB>>::decode(value)?;
            let decoded = Self::try_from(encoded)?;
            Ok(decoded)
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
            let mut encoded = Vec::<u8>::new();
            self.clone()
                .encode_to_stream(&mut encoded)
                .expect("cannot encode to SpatiaLite internal format");
            encoded.encode_by_ref(buf)
        }

        fn encode(
            self,
            buf: &mut <DB as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull
        where
            Self: Sized,
        {
            let mut encoded = Vec::<u8>::new();
            self.encode_to_stream(&mut encoded)
                .expect("cannot encode to SpatiaLite internal format");
            encoded.encode(buf)
        }
    }

    impl_geometry_sqlx_codecs!(SpatiaLite);
}

impl TryFrom<&[u8]> for SpatiaLiteGeometry {
    type Error = std::io::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(value);
        Self::decode_from_stream(&mut cursor)
    }
}

impl SpatiaLiteGeometry {
    pub fn encode_to_stream<W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.encode_to_stream_with_endianess::<NativeEndian, _>(stream)
    }

    pub fn encode_to_stream_with_endianess<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error>
    where
        E: ?Sized,
        Endianess: From<PhantomData<E>>,
    {
        // encode start byte, always 0x00
        stream.write_u8(0)?;
        // encode endianness
        stream.write_u8(Endianess::from(PhantomData::<E>).into())?;
        // encode SRID
        stream.write_u32::<E>(self.srid())?;
        // encode MBR
        self.mbr().encode_spatialite::<E, _>(stream)?;
        // encode geometry class
        self.kind().encode_spatialite::<E, _>(stream)?;

        match self.0 {
            Geometry::Point(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::LineString(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::Polygon(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::MultiPoint(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::MultiLineString(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::MultiPolygon(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::PointZ(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::LineStringZ(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::PolygonZ(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::MultiPointZ(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::MultiLineStringZ(a) => a.encode_spatialite::<E, _>(stream)?,
            Geometry::MultiPolygonZ(a) => a.encode_spatialite::<E, _>(stream)?,
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

    pub fn decode_from_stream_with_endianess<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        // Read the SRID
        let srid: u32 = stream.read_u32::<E>()?;

        // Read MBR
        let _mbr = MBR::decode_spatialite::<E, _>(stream)?;

        // Read the geometry class
        let kind = GeometryKind::decode_spatialite::<E, _>(stream)?;

        let mut geometry: Geometry = match kind {
            GeometryKind::Point => Point::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::LineString => LineString::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::Polygon => Polygon::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiPoint => MultiPoint::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiLineString => {
                MultiLineString::decode_spatialite::<E, _>(stream)?.into()
            }
            GeometryKind::MultiPolygon => MultiPolygon::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollection => todo!(),
            GeometryKind::PointZ => PointZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::LineStringZ => LineStringZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::PolygonZ => PolygonZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiPointZ => MultiPointZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::MultiLineStringZ => {
                MultiLineStringZ::decode_spatialite::<E, _>(stream)?.into()
            }
            GeometryKind::MultiPolygonZ => MultiPolygonZ::decode_spatialite::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollectionZ => todo!(),
        };

        geometry.set_srid(srid);

        let end = stream.read_u8()?;
        assert_eq!(end, 0xFE);

        Ok(Self(geometry))
    }
}

impl GeometryKind {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        let encoded = match self {
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
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
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
        }
        .into())
    }
}

impl MBR<f64> {
    pub fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        stream.write_f64::<E>(self.min_x)?;
        stream.write_f64::<E>(self.min_y)?;
        stream.write_f64::<E>(self.max_x)?;
        stream.write_f64::<E>(self.max_y)?;
        stream.write_u8(0x7C)?;

        Ok(())
    }
    pub fn decode_spatialite<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
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
}

impl<const N: usize> GenPoint<N, f64> {
    /// Encode un point dans un flux binaire.
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let coordinates = Vector::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenMultiPoint<N, f64> {
    /// Encode un ensemble de points dans un flux binaire
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let coordinates = VectorArray::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenLineString<N, f64> {
    /// Encode un ensemble de points dans un flux binaire
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let coordinates = VectorArray::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenMultiLineString<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenPolygon<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenMultiPolygon<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_spatialite::<E, _>(stream)
    }
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let coordinates = VectorTensor::<N, f64>::decode_spatialite::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> Vector<N, f64> {
    /// Encode un vecteur N-D
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        for i in 0..N {
            stream.write_f64::<E>(self[i])?;
        }

        Ok(())
    }

    /// Décode un vecteur n-D
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let mut coordinates: [f64; N] = [0f64; N];

        for i in 0..N {
            coordinates[i] = value.read_f64::<E>()?;
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorArray<N, f64> {
    /// Encode une liste de vecteurs dans le flux de sortie.
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
            vector.encode_spatialite::<E, _>(stream)?;
        }

        Ok(())
    }

    /// Décode une liste de vecteurs depuis un flux d'entrée.
    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let nb_points: u32 = stream.read_u32::<E>()?;
        let mut coordinates = Vec::<Vector<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(Vector::decode_spatialite::<E, _>(stream)?);
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorMatrix<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
            vector.encode_spatialite::<E, _>(stream)?;
        }

        Ok(())
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
        let nb_points: u32 = value.read_u32::<E>()?;
        let mut coordinates = Vec::<VectorArray<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(VectorArray::decode_spatialite::<E, _>(value)?);
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorTensor<N, f64> {
    pub(self) fn encode_spatialite<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
            vector.encode_spatialite::<E, _>(stream)?;
        }

        Ok(())
    }

    pub(self) fn decode_spatialite<E: ByteOrder, R: Read>(
        value: &mut R,
    ) -> Result<Self, std::io::Error> {
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

        let mut binary = Vec::<u8>::new();

        expected
            .clone()
            .encode_to_stream(&mut binary)
            .expect("cannot encode to stream");

        let mut stream = Cursor::new(binary);
        let value =
            SpatiaLiteGeometry::decode_from_stream(&mut stream).expect("cannot decode from stream");

        assert_eq!(value, expected)
    }
}
