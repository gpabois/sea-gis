#[cfg(feature = "sqlx")]
/// Implémente l'encodage / décodage depuis sqlx
mod sqlx {
    use crate::{
        ewkb::*,
        io::{Decodable, Encodable as _},
    };
    use ::sqlx::{postgres::PgTypeInfo, Database, Decode, Encode, Postgres, Type};

    impl Type<Postgres> for EWKBGeometry {
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
            let decoded = <Self as Decodable>::decode_from_slice(encoded).unwrap();
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
            let encoded = self.encode_to_vec().unwrap();
            encoded.encode_by_ref(buf)
        }
    }

    impl_geometry_sqlx_codecs!(EWKB);
}
