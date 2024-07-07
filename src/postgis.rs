use std::ops::{Deref, DerefMut};

use crate::types::Geometry;

#[derive(Debug, Clone, PartialEq)]
pub struct PgGeometry(Geometry);

impl PgGeometry {
    pub fn new<G: Into<Geometry>>(args: G) -> Self {
        Self(args.into())
    }
}

impl From<Geometry> for PgGeometry {
    fn from(value: Geometry) -> Self {
        Self(value)
    }
}

impl Deref for PgGeometry {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PgGeometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl_geometry_proxies!(Pg);

mod sqlx {
    use std::io::Cursor;

    use ::sqlx::{postgres::PgTypeInfo, Decode, Encode, Postgres, Type};

    use crate::ewkb::EWKBGeometry;

    use super::*;

    impl Type<Postgres> for PgGeometry {
        fn type_info() -> <Postgres as ::sqlx::Database>::TypeInfo {
            PgTypeInfo::with_name("geometry")
        }
    }

    impl<'r> Decode<'r, Postgres> for PgGeometry {
        fn decode(
            value: <Postgres as ::sqlx::database::HasValueRef<'r>>::ValueRef,
        ) -> Result<Self, ::sqlx::error::BoxDynError> {
            let ewkb = <EWKBGeometry as Decode<'r, Postgres>>::decode(value).unwrap();
            Ok(Self::new(ewkb))
        }
    }

    impl<'q> Encode<'q, Postgres> for PgGeometry {
        fn encode_by_ref(
            &self,
            buf: &mut <Postgres as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
        ) -> ::sqlx::encode::IsNull {
            let mut bytes: Vec<u8> = Vec::default();
            EWKBGeometry::encode_by_ref_to_stream(self.deref(), &mut bytes).unwrap();
            <Vec<u8> as Encode<'q, Postgres>>::encode_by_ref(&bytes, buf)
        }
    }

    impl_geometry_sqlx_codecs!(Pg);
}
