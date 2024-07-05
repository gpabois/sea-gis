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

mod sqlx {
    use std::marker::PhantomData;

    use ::sqlx::{Database, Decode, Encode};
    use sqlx::{Postgres, Sqlite};

    use crate::{ewkb::EWKBGeometry, spatialite::SpatiaLiteGeometry, types};

    use super::AutoGeometry;

    impl<'q, DB> Encode<'q, DB> for AutoGeometry
    where
        DB: Database,
        DatabaseKind: From<PhantomData<DB>>,
        Vec<u8>: Encode<'q, DB>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> sqlx::encode::IsNull {
            match DatabaseKind::new::<DB>() {
                DatabaseKind::Postgres => EWKBGeometry::from(self.0.clone()).encode_by_ref(buf),
                DatabaseKind::SqlLite => {
                    SpatiaLiteGeometry::from(self.0.clone()).encode_by_ref(buf)
                }
            }
        }
    }

    impl<'r, DB> Decode<'r, DB> for AutoGeometry
    where
        DB: Database,
        DatabaseKind: From<PhantomData<DB>>,
        &'r [u8]: Decode<'r, DB>,
    {
        fn decode(
            value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, sqlx::error::BoxDynError> {
            match DatabaseKind::new::<DB>() {
                DatabaseKind::Postgres => EWKBGeometry::decode(value)
                    .map(types::Geometry::from)
                    .map(Self::from),
                DatabaseKind::SqlLite => SpatiaLiteGeometry::decode(value)
                    .map(types::Geometry::from)
                    .map(Self::from),
            }
        }
    }

    enum DatabaseKind {
        Postgres,
        SqlLite,
    }

    impl DatabaseKind {
        pub fn new<DB: Database>() -> Self
        where
            Self: From<PhantomData<DB>>,
        {
            Self::from(PhantomData::<DB>)
        }
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
