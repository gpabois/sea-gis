macro_rules! impl_geometry_sqlx_codec {
    ($ns:ident, $geometry_type:ident) => {
        ::paste::paste! {
            impl<'r, DB> ::sqlx::Type<DB> for [<$ns $geometry_type>]
            where DB: ::sqlx::Database, [<$ns Geometry>]: ::sqlx::Type<DB>,
            {
                fn type_info() -> <DB as ::sqlx::Database>::TypeInfo {
                    [<$ns Geometry>]::type_info()
                }
            }

            impl<'r, DB> ::sqlx::Decode<'r, DB> for [<$ns $geometry_type>]
            where
                DB: ::sqlx::Database,
                [<$ns Geometry>]: ::sqlx::Decode<'r, DB>,
            {
                fn decode(
                    value: <DB as ::sqlx::database::HasValueRef<'r>>::ValueRef,
                ) -> Result<Self, ::sqlx::error::BoxDynError> {
                    let geom = <[<$ns Geometry>] as ::sqlx::Decode<'r, DB>>::decode(value)?.0;
                    Ok(Self(geom.try_into()?))
                }
            }

            impl<'q, DB> ::sqlx::Encode<'q, DB> for [<$ns $geometry_type>]
            where
                DB: ::sqlx::Database,
                [<$ns Geometry>]: ::sqlx::Encode<'q, DB>,
            {
                fn encode_by_ref(
                    &self,
                    buf: &mut <DB as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
                ) -> ::sqlx::encode::IsNull {
                    [<$ns Geometry>](self.0.clone().into()).encode_by_ref(buf)
                }
            }
        }
    };
}

macro_rules! impl_geometry_sqlx_codecs {
    ($ns: ident) => {
        impl_geometry_sqlx_codec!($ns, Point);
        impl_geometry_sqlx_codec!($ns, MultiPoint);
        impl_geometry_sqlx_codec!($ns, LineString);
        impl_geometry_sqlx_codec!($ns, MultiLineString);
        impl_geometry_sqlx_codec!($ns, Polygon);
        impl_geometry_sqlx_codec!($ns, MultiPolygon);

        impl_geometry_sqlx_codec!($ns, PointZ);
        impl_geometry_sqlx_codec!($ns, MultiPointZ);
        impl_geometry_sqlx_codec!($ns, LineStringZ);
        impl_geometry_sqlx_codec!($ns, MultiLineStringZ);
        impl_geometry_sqlx_codec!($ns, PolygonZ);
        impl_geometry_sqlx_codec!($ns, MultiPolygonZ);
    };
}

mod ewkb;

#[cfg(feature = "postgis")]
mod postgis;
#[cfg(feature = "spatialite")]
mod spatialite;

#[cfg(feature = "postgis")]
pub use postgis::*;

#[cfg(feature = "spatialite")]
pub use spatialite::*;
