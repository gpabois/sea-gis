use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use sea_query::{Nullable, ValueType};
use sea_orm::Value;

use crate::{ewkb, spatialite, types};

pub trait DatabaseNativeFormat {
    type Database: ::sqlx::Database;

    /// Objet pour encoder/décoder le format canonique de la base de données.
    type Type: ValueType 
        + Nullable 
        + Into<Value>
        + From<types::Geometry>
        + Into<types::Geometry>
        + for <'a> ::sqlx::Encode<'a, Self::Database>
        + for <'a> ::sqlx::Decode<'a, Self::Database>
        + Clone;
}

pub enum PostGIS{}

impl DatabaseNativeFormat for PostGIS {
    type Database = ::sqlx::Postgres;
    type Type = ewkb::EWKBGeometry;
}

pub enum SpatiaLite{}

impl DatabaseNativeFormat for SpatiaLite {
    type Database = ::sqlx::Sqlite;
    type Type = spatialite::SpatiaLiteGeometry;
}

#[derive(Clone)]
pub struct Geometry<F: DatabaseNativeFormat>(F::Type);

impl<F: DatabaseNativeFormat> Geometry<F> {
    pub fn new<G: Into<types::Geometry>>(value: G) -> Self {
        Self(value.into().into())
    }
}

impl<F: DatabaseNativeFormat> From<types::Geometry> for Geometry<F> {
    fn from(value: types::Geometry) -> Self {
        Self(value.into())
    }
}


pub struct PointS<D: DatabaseNativeFormat>{
    inner: types::PointS,
    _pht: PhantomData<D>
}

impl<F: DatabaseNativeFormat> Clone for PointS<F> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), _pht: self._pht.clone() }
    }
}

impl<F: DatabaseNativeFormat> PartialEq for PointS<F> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<F: DatabaseNativeFormat> std::fmt::Debug for PointS<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PointS").field("inner", &self.inner).finish()
    }
}

impl<D: DatabaseNativeFormat> PointS<D> {
    pub fn new<V: Into<types::Vector<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::PointS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::PointS> for PointS<D> {
    fn from(value: types::PointS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> Deref for PointS<D> {
    type Target = types::PointS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for PointS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPointS<D: DatabaseNativeFormat> {
    inner: types::MultiPointS,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> MultiPointS<D> {
    pub fn new<V: Into<types::VectorArray<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::MultiPointS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::MultiPointS> for MultiPointS<D> {
    fn from(value: types::MultiPointS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> Deref for MultiPointS<D> {
    type Target = types::MultiPointS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for MultiPointS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct LineStringS<D: DatabaseNativeFormat> {
    inner: types::LineStringS,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> LineStringS<D> {
    pub fn new<V: Into<types::VectorArray<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::LineStringS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::LineStringS> for LineStringS<D> {
    fn from(value: types::LineStringS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> Deref for LineStringS<D> {
    type Target = types::LineStringS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for LineStringS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiLineStringS<D: DatabaseNativeFormat>{
    inner: types::MultiLineStringS,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> MultiLineStringS<D> {
    pub fn new<V: Into<types::VectorMatrix<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::MultiLineStringS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::MultiLineStringS> for MultiLineStringS<D> {
    fn from(value: types::MultiLineStringS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> Deref for MultiLineStringS<D> {
    type Target = types::MultiLineStringS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for MultiLineStringS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PolygonS<D: DatabaseNativeFormat>{
    inner: types::PolygonS,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> From<types::PolygonS> for PolygonS<D> {
    fn from(value: types::PolygonS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}


impl<D: DatabaseNativeFormat> Deref for PolygonS<D> {
    type Target = types::PolygonS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for PolygonS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPolygonS<D: DatabaseNativeFormat>{
    inner: types::MultiPolygonS,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> Deref for MultiPolygonS<D> {
    type Target = types::MultiPolygonS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for MultiPolygonS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PointZ<D: DatabaseNativeFormat>{
    inner: types::PointZ,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> PointZ<D> {
    pub fn new<V: Into<types::Vector<3, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::PointZ::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: DatabaseNativeFormat> Deref for PointZ<D> {
    type Target = types::PointZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for PointZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPointZ<D: DatabaseNativeFormat>{
    inner: types::MultiPointZ,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> Deref for MultiPointZ<D> {
    type Target = types::MultiPointZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for MultiPointZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct LineStringZ<D: DatabaseNativeFormat>{
    inner: types::LineStringZ,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> Deref for LineStringZ<D> {
    type Target = types::LineStringZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for LineStringZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiLineStringZ<D: DatabaseNativeFormat>{
    inner: types::MultiLineStringZ,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> Deref for MultiLineStringZ<D> {
    type Target = types::MultiLineStringZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for MultiLineStringZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PolygonZ<D: DatabaseNativeFormat>{
    inner: types::PolygonZ,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> Deref for PolygonZ<D> {
    type Target = types::PolygonZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for PolygonZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPolygonZ<D: DatabaseNativeFormat>{
    inner: types::MultiPolygonZ,
    _pht: PhantomData<D>
}

impl<D: DatabaseNativeFormat> Deref for MultiPolygonZ<D> {
    type Target = types::MultiPolygonZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: DatabaseNativeFormat> DerefMut for MultiPolygonZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

mod sqlx {
    use super::*;
    use ::sqlx::{Database, Decode, Encode, Type};

    impl<'q, F> Encode<'q, F::Database> for Geometry<F> 
    where F: DatabaseNativeFormat
    {
        fn encode_by_ref(&self, buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> ::sqlx::encode::IsNull {
            self.0.clone().encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for Geometry<F> 
    where F: DatabaseNativeFormat
    {
        fn decode(value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, ::sqlx::error::BoxDynError> {
           let decoded = F::Type::decode(value)?;
           Ok(Self(decoded))
        }
    }

    impl<'q, F> Encode<'q, F::Database> for PointS<F> 
    where F: DatabaseNativeFormat
    {
        fn encode_by_ref(&self, buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for PointS<F> 
    where F: DatabaseNativeFormat
    {
        fn decode(value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, ::sqlx::error::BoxDynError> {
           let decoded = F::Type::decode(value)?;
           let geom: types::Geometry = decoded.into();
           let pt = types::PointS::try_from(geom)?;
           Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for PointS<F> where F: DatabaseNativeFormat, for <'a> &'a [u8]: Type<F::Database>
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
        
    }
}
