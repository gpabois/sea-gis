use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use sea_query::{Nullable, ValueType};
use sea_orm::Value;

use crate::{ewkb, spatialite, types, Vector, VectorArray, VectorMatrix};

pub trait Database {
    type CanonicalForm: ValueType 
        + Nullable 
        + Into<Value>
        + From<types::Geometry>;
}

pub enum PostGIS{}

impl Database for PostGIS {
    type CanonicalForm = ewkb::EWKBGeometry;
}

pub enum SpatiaLite{}

impl Database for SpatiaLite {
    type CanonicalForm = spatialite::SpatiaLiteGeometry;
}

pub struct Geometry<D: Database>(D::CanonicalForm);

impl<D: Database> From<types::Geometry> for Geometry<D> {
    fn from(value: types::Geometry) -> Self {
        Self(value.into())
    }
}

pub struct PointS<D: Database>{
    inner: types::PointS,
    _pht: PhantomData<D>
}

impl<D: Database> PointS<D> {
    pub fn new<V: Into<Vector<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::PointS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: Database> From<types::PointS> for PointS<D> {
    fn from(value: types::PointS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: Database> Deref for PointS<D> {
    type Target = types::PointS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for PointS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPointS<D: Database> {
    inner: types::MultiPointS,
    _pht: PhantomData<D>
}

impl<D: Database> MultiPointS<D> {
    pub fn new<V: Into<VectorArray<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::MultiPointS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: Database> From<types::MultiPointS> for MultiPointS<D> {
    fn from(value: types::MultiPointS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: Database> Deref for MultiPointS<D> {
    type Target = types::MultiPointS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for MultiPointS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct LineStringS<D: Database> {
    inner: types::LineStringS,
    _pht: PhantomData<D>
}

impl<D: Database> LineStringS<D> {
    pub fn new<V: Into<VectorArray<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::LineStringS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: Database> From<types::LineStringS> for LineStringS<D> {
    fn from(value: types::LineStringS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: Database> Deref for LineStringS<D> {
    type Target = types::LineStringS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for LineStringS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiLineStringS<D: Database>{
    inner: types::MultiLineStringS,
    _pht: PhantomData<D>
}

impl<D: Database> MultiLineStringS<D> {
    pub fn new<V: Into<VectorMatrix<2, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::MultiLineStringS::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: Database> From<types::MultiLineStringS> for MultiLineStringS<D> {
    fn from(value: types::MultiLineStringS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}

impl<D: Database> Deref for MultiLineStringS<D> {
    type Target = types::MultiLineStringS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for MultiLineStringS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PolygonS<D: Database>{
    inner: types::PolygonS,
    _pht: PhantomData<D>
}

impl<D: Database> From<types::PolygonS> for PolygonS<D> {
    fn from(value: types::PolygonS) -> Self {
        Self {
            inner: value,
            _pht: PhantomData
        }
    }
}


impl<D: Database> Deref for PolygonS<D> {
    type Target = types::PolygonS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for PolygonS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPolygonS<D: Database>{
    inner: types::MultiPolygonS,
    _pht: PhantomData<D>
}

impl<D: Database> Deref for MultiPolygonS<D> {
    type Target = types::MultiPolygonS;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for MultiPolygonS<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PointZ<D: Database>{
    inner: types::PointZ,
    _pht: PhantomData<D>
}

impl<D: Database> PointZ<D> {
    pub fn new<V: Into<Vector<3, f64>>>(coordinates: V) -> Self {
        Self {
            inner: types::PointZ::new(coordinates),
            _pht: PhantomData
        }
    }
}

impl<D: Database> Deref for PointZ<D> {
    type Target = types::PointZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for PointZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPointZ<D: Database>{
    inner: types::MultiPointZ,
    _pht: PhantomData<D>
}

impl<D: Database> Deref for MultiPointZ<D> {
    type Target = types::MultiPointZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for MultiPointZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct LineStringZ<D: Database>{
    inner: types::LineStringZ,
    _pht: PhantomData<D>
}

impl<D: Database> Deref for LineStringZ<D> {
    type Target = types::LineStringZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for LineStringZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiLineStringZ<D: Database>{
    inner: types::MultiLineStringZ,
    _pht: PhantomData<D>
}

impl<D: Database> Deref for MultiLineStringZ<D> {
    type Target = types::MultiLineStringZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for MultiLineStringZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PolygonZ<D: Database>{
    inner: types::PolygonZ,
    _pht: PhantomData<D>
}

impl<D: Database> Deref for PolygonZ<D> {
    type Target = types::PolygonZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for PolygonZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct MultiPolygonZ<D: Database>{
    inner: types::MultiPolygonZ,
    _pht: PhantomData<D>
}

impl<D: Database> Deref for MultiPolygonZ<D> {
    type Target = types::MultiPolygonZ;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Database> DerefMut for MultiPolygonZ<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}