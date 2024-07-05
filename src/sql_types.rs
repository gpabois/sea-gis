//! Ce module contient les types qui peuvent être encodés/décodés depuis
//! les base de données pris en charge par ce crate.
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use crate::ewkb::EWKBGeometry;
pub use crate::spatialite::SpatiaLiteGeometry;
pub use crate::auto::AutoGeometry;

pub type PgGeometry = EWKBGeometry;