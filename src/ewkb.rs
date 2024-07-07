//! Module contenant les objets permettant d'encoder/décoder au format Extended Well-Known Bytes
//! (EWKB)
//!
//! Voir [self::EWKBGeometry]
use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};
use std::{
    io::{Cursor, Read, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::types::{
    GenLineString, GenMultiLineString, GenMultiPoint, GenMultiPolygon, GenPoint, GenPolygon,
    LineString, LineStringZ, MultiLineString, MultiLineStringZ, MultiPoint, MultiPointZ,
    MultiPolygon, MultiPolygonZ, Point, PointZ, Polygon, PolygonZ, Vector, VectorArray,
    VectorMatrix, VectorTensor,
};

use super::types::{Geometry, GeometryKind};

/// Objet intermédiaire pour encoder/decoder
/// au format EWKB toute géométrie.
#[derive(Debug, Clone, PartialEq)]
pub struct EWKBGeometry(Geometry);

impl TryFrom<&[u8]> for EWKBGeometry {
    type Error = std::io::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(value);
        Self::decode_from_stream(&mut cursor)
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

    /// Encode une géométrie au format EWKB dans le flux de sortie.
    ///
    /// Utilise par défaut le boutisme natif.
    pub fn encode_to_stream<W: Write>(self, stream: &mut W) -> Result<(), std::io::Error> {
        self.encode_to_stream_with_endianess::<NativeEndian, _>(stream)
    }

    /// Encode une géométrie au format EWKB dans le flux de sortie, avec un boutisme défini.
    pub fn encode_to_stream_with_endianess<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error>
    where
        Endianess: From<PhantomData<E>>,
    {
        // Write endianness.
        stream.write_u8(Endianess::from(PhantomData::<E>).into())?;

        // Write the EWKB type
        self.kind().encode_ewkb::<E, _>(stream)?;

        // Write the SRID
        stream.write_u32::<E>(self.srid())?;

        match self.0 {
            Geometry::Point(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::LineString(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::Polygon(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPoint(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiLineString(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPolygon(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::PointZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::LineStringZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::PolygonZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPointZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiLineStringZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPolygonZ(a) => a.encode_ewkb::<E, _>(stream),
        }
    }

    /// Encode une géométrie au format EWKB dans le flux de sortie.
    ///
    /// Utilise par défaut le boutisme natif.
    pub fn encode_by_ref_to_stream<W: Write>(
        geometry: &Geometry,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        Self::encode_by_ref_to_stream_with_endianess::<NativeEndian, _>(geometry, stream)
    }

    /// Encode une géométrie au format EWKB dans le flux de sortie, avec un boutisme défini.
    pub fn encode_by_ref_to_stream_with_endianess<E: ByteOrder, W: Write>(
        geometry: &Geometry,
        stream: &mut W,
    ) -> Result<(), std::io::Error>
    where
        Endianess: From<PhantomData<E>>,
    {
        // Write endianness.
        stream.write_u8(Endianess::from(PhantomData::<E>).into())?;

        // Write the EWKB type
        geometry.kind().encode_ewkb::<E, _>(stream)?;

        // Write the SRID
        stream.write_u32::<E>(geometry.srid())?;

        match geometry {
            Geometry::Point(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::LineString(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::Polygon(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPoint(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiLineString(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPolygon(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::PointZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::LineStringZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::PolygonZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPointZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiLineStringZ(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPolygonZ(a) => a.encode_ewkb::<E, _>(stream),
        }
    }

    /// Décode une géométrie encodée en EWKB
    pub fn decode_from_stream<R: Read>(buf: &mut R) -> Result<Self, std::io::Error> {
        // 0: Big Endian, 1: Little Endian
        let endianess = buf.read_u8()?;

        if endianess == BIG_ENDIAN {
            Self::decode_from_stream_with_endianess::<BigEndian, _>(buf)
        } else {
            Self::decode_from_stream_with_endianess::<LittleEndian, _>(buf)
        }
    }

    /// Décode une géométrie avec un boutisme défini
    pub fn decode_from_stream_with_endianess<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let kind: GeometryKind = GeometryKind::decode_ewkb::<E, _>(stream)?;
        let srid: u32 = stream.read_u32::<E>()?;

        let mut geometry: Geometry = match kind {
            GeometryKind::Point => Point::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::LineString => LineString::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::Polygon => Polygon::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiPoint => MultiPoint::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiLineString => MultiLineString::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiPolygon => MultiPolygon::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollection => todo!(),
            GeometryKind::PointZ => PointZ::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::LineStringZ => LineStringZ::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::PolygonZ => PolygonZ::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiPointZ => MultiPointZ::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiLineStringZ => MultiLineStringZ::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiPolygonZ => MultiPolygonZ::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollectionZ => todo!(),
        };

        geometry.set_srid(srid);

        Ok(EWKBGeometry(geometry))
    }
}

impl GeometryKind {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
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

            GeometryKind::PointZ => 0x80000001,
            GeometryKind::LineStringZ => 0x80000002,
            GeometryKind::PolygonZ => 0x80000003,
            GeometryKind::MultiPointZ => 0x80000004,
            GeometryKind::MultiLineStringZ => 0x80000005,
            GeometryKind::MultiPolygonZ => 0x80000006,
            GeometryKind::GeometryCollectionZ => 0x80000007,
        };

        stream.write_u32::<E>(encoded | 0x20000000)
    }

    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let kind = match stream.read_u32::<E>()? & !0x20000000 {
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

            _ => panic!("invalid EWKB geometry"),
        };

        Ok(kind)
    }
}

impl_geometry_proxies!(EWKB);

const BIG_ENDIAN: u8 = 0;
const LITTLE_ENDIAN: u8 = 1;

impl<const N: usize> GenPoint<N, f64> {
    /// Encode un point dans un flux binaire.
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = Vector::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenMultiPoint<N, f64> {
    /// Encode un ensemble de Point dans un flux binaire
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorArray::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenLineString<N, f64> {
    /// Encode un ensemble de Point dans un flux binaire
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }

    /// Décode un point depuis un flux binaire
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorArray::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenMultiLineString<N, f64> {
    /// Encode dans un flux binaire
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }

    /// Décode depuis un flux binaire
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenPolygon<N, f64> {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }

    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> GenMultiPolygon<N, f64> {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorTensor::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> Vector<N, f64> {
    /// Encode un vecteur N-D
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        for i in 0..N {
            stream.write_f64::<E>(self[i])?;
        }

        Ok(())
    }

    /// Décode un vecteur n-D
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let mut coordinates: [f64; N] = [0f64; N];

        for i in 0..N {
            coordinates[i] = value.read_f64::<E>()?;
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorArray<N, f64> {
    /// Encode une liste de vecteurs dans le flux de sortie.
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of Point.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.iter() {
            vector.encode_ewkb::<E, _>(stream)?;
        }

        Ok(())
    }

    /// Décode une liste de vecteurs depuis un flux d'entrée.
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let nb_points: u32 = stream.read_u32::<E>()?;
        let mut coordinates = Vec::<Vector<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(Vector::decode_ewkb::<E, _>(stream)?);
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorMatrix<N, f64> {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of Point.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.iter() {
            vector.encode_ewkb::<E, _>(stream)?;
        }

        Ok(())
    }

    /// Décode un vecteur n-D
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let nb_points: u32 = stream.read_u32::<E>()?;
        let mut coordinates = Vec::<VectorArray<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(VectorArray::<N, f64>::decode_ewkb::<E, _>(stream)?);
        }

        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> VectorTensor<N, f64> {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        &self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of Point.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.iter() {
            vector.encode_ewkb::<E, _>(stream)?;
        }

        Ok(())
    }
    /// Décode un vecteur n-D
    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let nb_points: u32 = stream.read_u32::<E>()?;
        let mut coordinates = Vec::<VectorMatrix<N, f64>>::with_capacity(nb_points as usize);

        for _ in 0..nb_points {
            coordinates.push(VectorMatrix::<N, f64>::decode_ewkb::<E, _>(stream)?);
        }

        Ok(Self::new(coordinates))
    }
}

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

/// Implémente l'encodage / décodage depuis sqlx
mod sqlx {
    use super::*;
    use ::sqlx::{postgres::PgTypeInfo, Database, Decode, Encode, Postgres, Type};

    impl<'r> Type<Postgres> for EWKBGeometry {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_name("geometry")
        }
    }

    impl<'r, DB> Decode<'r, DB> for EWKBGeometry
    where
        DB: Database,
        &'r [u8]: Decode<'r, DB>,
    {
        fn decode(
            value: <DB as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let encoded = <&'r [u8] as Decode<DB>>::decode(value)?;
            println!("{:?}", encoded);
            let decoded = Self::try_from(encoded)?;
            Ok(decoded)
        }
    }

    impl<'q, DB> Encode<'q, DB> for EWKBGeometry
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
                .expect("cannot decode EWKB");
            println!("{:?}", encoded);
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
                .expect("cannot encode EWKB");
            encoded.encode(buf)
        }
    }

    impl_geometry_sqlx_codecs!(EWKB);
}

/// Implémente l'encodage / décodage pour Sea ORM
mod sea_orm {
    use super::*;

    use sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType};

    impl From<EWKBGeometry> for Value {
        fn from(value: EWKBGeometry) -> Self {
            let mut buf = Vec::<u8>::default();
            value
                .encode_to_stream(&mut buf)
                .expect("cannot encode EWKB geometry");
            buf.into()
        }
    }
    impl Nullable for EWKBGeometry {
        fn null() -> sea_orm::Value {
            sea_orm::Value::Bytes(None)
        }
    }

    impl ValueType for EWKBGeometry {
        fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
            match v {
                Value::Bytes(Some(boxed_buf)) => {
                    let mut buf = Cursor::new(boxed_buf.as_ref());
                    EWKBGeometry::decode_from_stream(&mut buf).map_err(|_| sea_query::ValueTypeErr)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_isomorphism_ewkb() {
        let expected = EWKBGeometry::from(Geometry::from(Point::new([10.0, 20.0])));

        let mut binary = Vec::<u8>::new();

        expected
            .clone()
            .encode_to_stream(&mut binary)
            .expect("cannot encode to stream");

        let mut stream = Cursor::new(binary);
        let value =
            EWKBGeometry::decode_from_stream(&mut stream).expect("cannot decode from stream");

        assert_eq!(value, expected)
    }
}
