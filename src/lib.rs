macro_rules! impl_geometry_proxy {
    ($ns:ident, $geometry_type:ident) => {
        ::paste::paste! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct [<$ns $geometry_type>] (pub(crate) crate::types::$geometry_type);

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

            impl std::ops::Deref for [<$ns $geometry_type>] {
                type Target = crate::types::$geometry_type;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }


            impl ::std::ops::DerefMut for [<$ns $geometry_type>] {
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

mod io;

pub mod error;
pub mod ewkb;

#[cfg(feature = "geojson")]
pub mod geojson;

#[cfg(feature = "sql")]
pub mod sql_types;

pub mod types;

const DEFAULT_SRID: u32 = 4326;
