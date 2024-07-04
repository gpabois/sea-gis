//! Ce module contient les types qui peuvent être encodés/décodés depuis
//! les base de données pris en charge par ce crate.
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use sea_orm::Value;
use sea_query::{Nullable, ValueType};

use crate::{
    ewkb, spatialite,
    types::{self, VectorMatrix},
};

pub trait DatabaseNativeFormat {
    type Database: ::sqlx::Database;

    /// Objet pour encoder/décoder le format canonique de la base de données.
    type Type: ValueType
        + Nullable
        + Into<Value>
        + From<types::Geometry>
        + Into<types::Geometry>
        + for<'a> ::sqlx::Encode<'a, Self::Database>
        + for<'a> ::sqlx::Decode<'a, Self::Database>
        + Clone;
}

pub enum PostGIS {}

impl DatabaseNativeFormat for PostGIS {
    type Database = ::sqlx::Postgres;
    type Type = ewkb::EWKBGeometry;
}

pub enum SpatiaLite {}

impl DatabaseNativeFormat for SpatiaLite {
    type Database = ::sqlx::Sqlite;
    type Type = spatialite::SpatiaLiteGeometry;
}

/// Représente une géométrie qui peut être encoder/décoder depuis
/// une base de donnée.
///
/// Deux formats sont possibles :
/// - PostGIS (voir [crate::ewkb::EWKBGeometry]) ;
/// - SpatiaLite (voir [crate::spatialite::SpatiaLiteGeometry])
///
/// # Exemple
/// ```
/// use sql_gis::sql_types::{Geometry, PostGIS};
///
/// let geom = Geometry::<PostGIS>::new_point_s([1.0, 2.0]);
/// ```
#[derive(Clone)]
pub struct Geometry<F: DatabaseNativeFormat>(F::Type);

impl<F: DatabaseNativeFormat> From<Geometry<F>> for Value {
    fn from(value: Geometry<F>) -> Self {
        value.0.into()
    }
}

impl<F: DatabaseNativeFormat> Geometry<F> {
    pub fn new<G: Into<types::Geometry>>(value: G) -> Self {
        Self(value.into().into())
    }

    /// Crée une nouvelle géométrie qui correspond à un PointS.
    pub fn new_point_s<G: Into<types::Vector<2, f64>>>(coordinates: G) -> Self {
        Self::new(types::PointS::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un MultiPointS.
    pub fn new_multi_point_s<G: Into<types::VectorArray<2, f64>>>(coordinates: G) -> Self {
        Self::new(types::MultiPointS::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à une LineStringS.
    pub fn new_line_string_s<G: Into<types::VectorArray<2, f64>>>(coordinates: G) -> Self {
        Self::new(types::LineStringS::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à une MultiLineStringS.
    pub fn new_multi_line_string_s<G: Into<types::VectorMatrix<2, f64>>>(coordinates: G) -> Self {
        Self::new(types::MultiLineStringS::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un PolygonS.
    pub fn new_polygon_s<G: Into<types::VectorMatrix<2, f64>>>(coordinates: G) -> Self {
        Self::new(types::PolygonS::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un MultiPolygonS.
    pub fn new_multi_polygon_s<G: Into<types::VectorTensor<2, f64>>>(coordinates: G) -> Self {
        Self::new(types::MultiPolygonS::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un PointZ.
    pub fn new_point_z<G: Into<types::Vector<3, f64>>>(coordinates: G) -> Self {
        Self::new(types::PointZ::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un MultiPointZ.
    pub fn new_multi_point_z<G: Into<types::VectorArray<3, f64>>>(coordinates: G) -> Self {
        Self::new(types::MultiPointZ::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à une LineStringZ.
    pub fn new_line_string_z<G: Into<types::VectorArray<3, f64>>>(coordinates: G) -> Self {
        Self::new(types::LineStringZ::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à une MultiLineStringZ.
    pub fn new_multi_line_string_z<G: Into<types::VectorMatrix<3, f64>>>(coordinates: G) -> Self {
        Self::new(types::MultiLineStringZ::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un PolygonZ.
    pub fn new_polygon_z<G: Into<types::VectorMatrix<3, f64>>>(coordinates: G) -> Self {
        Self::new(types::PolygonZ::new(coordinates))
    }

    /// Crée une nouvelle géométrie qui correspond à un MultiPolygonZ.
    pub fn new_multi_polygon_z<G: Into<types::VectorTensor<3, f64>>>(coordinates: G) -> Self {
        Self::new(types::MultiPolygonZ::new(coordinates))
    }
}

impl<F: DatabaseNativeFormat> From<types::Geometry> for Geometry<F> {
    fn from(value: types::Geometry) -> Self {
        Self(value.into())
    }
}

/// Point 2D (voir [crate::types::PointS])
pub struct PointS<D: DatabaseNativeFormat> {
    inner: types::PointS,
    _pht: PhantomData<D>,
}

impl<F: DatabaseNativeFormat> Clone for PointS<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _pht: self._pht.clone(),
        }
    }
}

impl<F: DatabaseNativeFormat> PartialEq for PointS<F> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<F: DatabaseNativeFormat> std::fmt::Debug for PointS<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PointS")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<D: DatabaseNativeFormat> PointS<D> {
    pub fn new<V: Into<types::Vector<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::PointS::new(coordinates),
            _pht: PhantomData,
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::PointS> for PointS<D> {
    fn from(value: types::PointS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
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
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> MultiPointS<D> {
    pub fn new<V: Into<types::VectorArray<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::MultiPointS::new(coordinates),
            _pht: PhantomData,
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::MultiPointS> for MultiPointS<D> {
    fn from(value: types::MultiPointS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
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
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> LineStringS<D> {
    pub fn new<V: Into<types::VectorArray<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::LineStringS::new(coordinates),
            _pht: PhantomData,
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::LineStringS> for LineStringS<D> {
    fn from(value: types::LineStringS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
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

pub struct MultiLineStringS<D: DatabaseNativeFormat> {
    inner: types::MultiLineStringS,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> MultiLineStringS<D> {
    pub fn new<V: Into<types::VectorMatrix<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::MultiLineStringS::new(coordinates),
            _pht: PhantomData,
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::MultiLineStringS> for MultiLineStringS<D> {
    fn from(value: types::MultiLineStringS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
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

pub struct PolygonS<D: DatabaseNativeFormat> {
    inner: types::PolygonS,
    _pht: PhantomData<D>,
}

impl<F: DatabaseNativeFormat> PolygonS<F> {
    pub fn new<V: Into<VectorMatrix<2, f64>>>(args: V) -> Self {
        Self {
            inner: types::PolygonS::new(args),
            _pht: PhantomData,
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::PolygonS> for PolygonS<D> {
    fn from(value: types::PolygonS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
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

pub struct MultiPolygonS<D: DatabaseNativeFormat> {
    inner: types::MultiPolygonS,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> From<types::MultiPolygonS> for MultiPolygonS<D> {
    fn from(value: types::MultiPolygonS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
        }
    }
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

pub struct PointZ<D: DatabaseNativeFormat> {
    inner: types::PointZ,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> PointZ<D> {
    pub fn new<V: Into<types::Vector<3, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::PointZ::new(coordinates),
            _pht: PhantomData,
        }
    }
}

impl<D: DatabaseNativeFormat> From<types::PointZ> for PointZ<D> {
    fn from(value: types::PointZ) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
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

pub struct MultiPointZ<D: DatabaseNativeFormat> {
    inner: types::MultiPointZ,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> From<types::MultiPointZ> for MultiPointZ<D> {
    fn from(value: types::MultiPointZ) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
        }
    }
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

pub struct LineStringZ<D: DatabaseNativeFormat> {
    inner: types::LineStringZ,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> From<types::LineStringZ> for LineStringZ<D> {
    fn from(value: types::LineStringZ) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
        }
    }
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

pub struct MultiLineStringZ<D: DatabaseNativeFormat> {
    inner: types::MultiLineStringZ,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> From<types::MultiLineStringZ> for MultiLineStringZ<D> {
    fn from(value: types::MultiLineStringZ) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
        }
    }
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

pub struct PolygonZ<D: DatabaseNativeFormat> {
    inner: types::PolygonZ,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> From<types::PolygonZ> for PolygonZ<D> {
    fn from(value: types::PolygonZ) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
        }
    }
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

pub struct MultiPolygonZ<D: DatabaseNativeFormat> {
    inner: types::MultiPolygonZ,
    _pht: PhantomData<D>,
}

impl<D: DatabaseNativeFormat> From<types::MultiPolygonZ> for MultiPolygonZ<D> {
    fn from(value: types::MultiPolygonZ) -> Self {
        Self {
            inner: value,
            _pht: PhantomData,
        }
    }
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
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            self.0.clone().encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for Geometry<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            Ok(Self(decoded))
        }
    }

    impl<'q, F> Encode<'q, F::Database> for PointS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for PointS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::PointS::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for PointS<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for MultiPointS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for MultiPointS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::MultiPointS::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for MultiPointS<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for LineStringS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for LineStringS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::LineStringS::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for LineStringS<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for MultiLineStringS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for MultiLineStringS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::MultiLineStringS::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for MultiLineStringS<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for PolygonS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for PolygonS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::PolygonS::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for PolygonS<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for MultiPolygonS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for MultiPolygonS<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::MultiPolygonS::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for MultiPolygonS<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for PointZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for PointZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::PointZ::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for PointZ<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for MultiPointZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for MultiPointZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::MultiPointZ::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for MultiPointZ<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for LineStringZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for LineStringZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::LineStringZ::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for LineStringZ<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for MultiLineStringZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for MultiLineStringZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::MultiLineStringZ::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for MultiLineStringZ<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for PolygonZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for PolygonZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::PolygonZ::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for PolygonZ<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }

    impl<'q, F> Encode<'q, F::Database> for MultiPolygonZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <F::Database as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            Geometry::<F>::new(self.inner.clone()).encode_by_ref(buf)
        }
    }

    impl<'r, F> Decode<'r, F::Database> for MultiPolygonZ<F>
    where
        F: DatabaseNativeFormat,
    {
        fn decode(
            value: <F::Database as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let decoded = F::Type::decode(value)?;
            let geom: types::Geometry = decoded.into();
            let pt = types::MultiPolygonZ::try_from(geom)?;
            Ok(Self::from(pt))
        }
    }

    impl<F> Type<F::Database> for MultiPolygonZ<F>
    where
        F: DatabaseNativeFormat,
        for<'a> &'a [u8]: Type<F::Database>,
    {
        fn type_info() -> <F::Database as Database>::TypeInfo {
            <&[u8] as Type<F::Database>>::type_info()
        }
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::Iden;
    use sea_query::{PostgresQueryBuilder, Query, QueryStatementWriter};

    use super::{Geometry, PostGIS};

    #[derive(Iden)]
    pub enum GisIden {
        Table,
        Geometry,
    }

    /// Vérifie que le type SQL Géométrie peut être injectée en tant que valeur
    /// d'une requête sea-query.
    #[test]
    pub fn test_sea_query_geometry_value() {
        let q = Query::insert()
            .into_table(GisIden::Table)
            .columns([GisIden::Geometry])
            .values([Geometry::<PostGIS>::new_point_s([0.1, 0.2]).into()])
            .expect("cannot create query")
            .to_owned();

        let query = q.to_string(PostgresQueryBuilder);

        println!("{query}");
    }
}
