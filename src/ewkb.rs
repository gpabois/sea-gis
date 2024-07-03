use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian, ReadBytesExt, WriteBytesExt};

use std::{
    io::{Cursor, Read, Write},
    ops::{Deref, DerefMut},
};

use super::types::{
    Geometry, GeometryKind, LineString, LineStringS, LineStringZ, MultiLineString,
    MultiLineStringS, MultiLineStringZ, MultiPoint, MultiPointS, MultiPointZ, MultiPolygon,
    MultiPolygonS, MultiPolygonZ, Point, PointZ, PointS, Polygon, PolygonS, PolygonZ, Vector, VectorArray,
    VectorMatrix, VectorTensor,
};

/// Objet intermédiaire pour encoder/decoder
/// au format EWKB toute géométrie.
#[derive(Clone, PartialEq)]
pub struct EWKBGeometry(Geometry);

/// Implémente l'encodage / décodage depuis sqlx
mod sqlx {
    use sqlx::{Database, Decode, Encode};
    use super::EWKBGeometry;


    impl<'r, DB> Decode<'r,DB> for EWKBGeometry 
    where DB: Database, &'r [u8]: Decode<'r, DB>
    {
        fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, sqlx::error::BoxDynError> {
            let encoded = <&'r [u8] as Decode<DB>>::decode(value)?;
            let decoded = Self::try_from(encoded)?;
            Ok(decoded)
        }
    } 

    impl<'q, DB> Encode<'q, DB> for EWKBGeometry 
    where DB: Database, Vec<u8>: Encode<'q, DB>
    {
        fn encode_by_ref(&self, buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
            let mut encoded = Vec::<u8>::new();
            self.clone().encode_to_stream(&mut encoded).expect("cannot encode EWKB");
            encoded.encode_by_ref(buf)
        }
        
        fn encode(self, buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull
        where
            Self: Sized,
        {
            let mut encoded = Vec::<u8>::new();
            self.encode_to_stream(&mut encoded).expect("cannot encode EWKB");
            encoded.encode(buf)
        }
    }
}

/// Implémente l'encodage / décodage pour Sea ORM
mod sea_orm {
    use super::*;
    
    use sea_query::{Value, ValueType, Nullable, ColumnType, ArrayType};

    impl From<EWKBGeometry> for Value {
        fn from(value: EWKBGeometry) -> Self {
            let mut buf = Vec::<u8>::default();
            value.encode_to_stream(&mut buf).expect("cannot encode EWKB geometry");
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
        Endianess: From<E>,
    {
        // Write endianness.
        stream.write_u8(Endianess::from(E::default()).into())?;

        // Write the EWKB type
        self.kind().encode_ewkb::<E, _>(stream)?;

        // Write the SRID
        stream.write_u32::<E>(self.srid())?;

        match self.0 {
            Geometry::PointS(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::LineStringS(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::PolygonS(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPointS(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiLineStringS(a) => a.encode_ewkb::<E, _>(stream),
            Geometry::MultiPolygonS(a) => a.encode_ewkb::<E, _>(stream),
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
            GeometryKind::PointS => PointS::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::LineStringS => LineStringS::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::PolygonS => PolygonS::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiPointS => MultiPointS::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiLineStringS => MultiLineStringS::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::MultiPolygonS => MultiPolygonS::decode_ewkb::<E, _>(stream)?.into(),
            GeometryKind::GeometryCollectionS => todo!(),
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
            GeometryKind::PointS => 1,
            GeometryKind::LineStringS => 2,
            GeometryKind::PolygonS => 3,
            GeometryKind::MultiPointS => 4,
            GeometryKind::MultiLineStringS => 5,
            GeometryKind::MultiPolygonS => 6,
            GeometryKind::GeometryCollectionS => 7,

            GeometryKind::PointZ => 0x80000001,
            GeometryKind::LineStringZ => 0x80000002,
            GeometryKind::PolygonZ => 0x80000003,
            GeometryKind::MultiPointZ => 0x80000004,
            GeometryKind::MultiLineStringZ => 0x80000005,
            GeometryKind::MultiPolygonZ => 0x80000006,
            GeometryKind::GeometryCollectionZ => 0x80000007,
        };

        stream.write_u32::<E>(encoded)
    }

    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(
        stream: &mut R,
    ) -> Result<Self, std::io::Error> {
        let kind = match stream.read_u32::<E>()? {
            1 => GeometryKind::PointS,
            2 => GeometryKind::LineStringS,
            3 => GeometryKind::PolygonS,
            4 => GeometryKind::MultiPointS,
            5 => GeometryKind::MultiLineStringS,
            6 => GeometryKind::MultiPolygonS,
            7 => GeometryKind::GeometryCollectionS,

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

const BIG_ENDIAN: u8 = 0;
const LITTLE_ENDIAN: u8 = 1;

impl<const N: usize> Point<N, f64> {
    /// Encode un point dans un flux binaire.
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        self,
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

impl<const N: usize> MultiPoint<N, f64> {
    /// Encode un ensemble de points dans un flux binaire
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        self,
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

impl<const N: usize> LineString<N, f64> {
    /// Encode un ensemble de points dans un flux binaire
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        self,
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

impl<const N: usize> MultiLineString<N, f64> {
    /// Encode dans un flux binaire
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        self,
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

impl<const N: usize> Polygon<N, f64> {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        self.coordinates.encode_ewkb::<E, _>(stream)
    }

    pub(self) fn decode_ewkb<E: ByteOrder, R: Read>(value: &mut R) -> Result<Self, std::io::Error> {
        let coordinates = VectorMatrix::<N, f64>::decode_ewkb::<E, _>(value)?;
        Ok(Self::new(coordinates))
    }
}

impl<const N: usize> MultiPolygon<N, f64> {
    pub(self) fn encode_ewkb<E: ByteOrder, W: Write>(
        self,
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
        self,
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
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
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
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
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
        self,
        stream: &mut W,
    ) -> Result<(), std::io::Error> {
        // Write number of points.
        stream.write_u32::<E>(self.len() as u32)?;

        for vector in self.into_iter() {
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
