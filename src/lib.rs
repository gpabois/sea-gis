pub mod sql_types;
pub mod types;

pub mod error;
pub mod functions;

macro_rules! impl_geometry_proxy {
    ($ns:ident, $geometry_type:ident) => {
        ::paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct [<$ns $geometry_type>] (crate::types::$geometry_type);

            impl From<[<$ns $geometry_type>]> for crate::types::Geometry  {
                fn from(value: [<$ns $geometry_type>]) -> Self {
                    value.0.into()
                }
            }

            impl From<crate::types::$geometry_type> for [<$ns $geometry_type>] {
                fn from(value: crate::types::$geometry_type) -> Self {
                    Self(value)
                }
            }

            impl From<[<$ns $geometry_type>]> for crate::types::$geometry_type  {
                fn from(value: [<$ns $geometry_type>]) -> Self {
                    value.0
                }
            }

            impl Deref for [<$ns $geometry_type>] {
                type Target = crate::types::$geometry_type;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }


            impl DerefMut for [<$ns $geometry_type>] {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        }
    };
}

macro_rules! impl_geometry_proxies {
    ($ns: ident) => {
        impl_geometry_proxy!($ns, Point);
        impl_geometry_proxy!($ns, MultiPoint);
        impl_geometry_proxy!($ns, LineString);
        impl_geometry_proxy!($ns, MultiLineString);
        impl_geometry_proxy!($ns, Polygon);
        impl_geometry_proxy!($ns, MultiPolygon);

        impl_geometry_proxy!($ns, PointZ);
        impl_geometry_proxy!($ns, MultiPointZ);
        impl_geometry_proxy!($ns, LineStringZ);
        impl_geometry_proxy!($ns, MultiLineStringZ);
        impl_geometry_proxy!($ns, PolygonZ);
        impl_geometry_proxy!($ns, MultiPolygonZ);
    };
}

macro_rules! impl_geometry_sqlx_codec {
    ($ns:ident, $geometry_type:ident) => {
        ::paste::paste! {
            impl<'r, DB> ::sqlx::Type<DB> for [<$ns $geometry_type>]
            where DB: ::sqlx::Database, [<$ns Geometry>]: ::sqlx::Type<DB>,
            {
                fn type_info() -> <DB as Database>::TypeInfo {
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
                    let geom = [<$ns Geometry>]::decode(value)?.0;
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

mod auto;
pub mod ewkb;
pub mod geo_json;
mod postgis;
mod spatialite;
