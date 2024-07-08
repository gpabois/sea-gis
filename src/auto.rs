use std::ops::{Deref, DerefMut};

use crate::types;

/// Objet intermédiaire détectant le type de base de données
/// pour appliquer le codec adéquat.
pub struct AutoGeometry(types::Geometry);

impl Deref for AutoGeometry {
    type Target = types::Geometry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AutoGeometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<types::Geometry> for AutoGeometry {
    fn from(value: types::Geometry) -> Self {
        Self(value)
    }
}

impl From<AutoGeometry> for types::Geometry {
    fn from(value: AutoGeometry) -> Self {
        value.0
    }
}

impl_geometry_proxies!(Auto);

mod sqlx {
    use std::fmt::write;
    use std::marker::PhantomData;

    use ::sqlx::{Database, Decode, Encode, Type};
    use ::sqlx::{Postgres, Sqlite};

    use crate::{postgis::PgGeometry, spatialite::SpatiaLiteGeometry, types};

    use super::*;

    impl<'r, DB> Type<DB> for AutoGeometry
    where
        DB: Database,
        &'r [u8]: Type<DB>,
    {
        fn type_info() -> <DB as Database>::TypeInfo {
            <&[u8]>::type_info()
        }
    }

    impl<'q> Encode<'q, Postgres> for AutoGeometry {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            PgGeometry::new(self.0.clone()).encode_by_ref(buf)
        }
    }
    impl<'q> Encode<'q, Sqlite> for AutoGeometry {
        fn encode_by_ref(
            &self,
            buf: &mut <Sqlite as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            <SpatiaLiteGeometry as Encode<'q, Sqlite>>::encode_by_ref(
                &SpatiaLiteGeometry::new(self.0.clone()),
                buf,
            )
        }
    }
    impl_geometry_sqlx_codecs!(Auto);

    enum DatabaseKind {
        Postgres,
        SqlLite,
    }

    impl From<PhantomData<Postgres>> for DatabaseKind {
        fn from(_: PhantomData<Postgres>) -> Self {
            DatabaseKind::Postgres
        }
    }

    impl From<PhantomData<Sqlite>> for DatabaseKind {
        fn from(_: PhantomData<Sqlite>) -> Self {
            DatabaseKind::SqlLite
        }
    }
}
