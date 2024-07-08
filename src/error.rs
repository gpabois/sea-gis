use core::fmt;

use super::types::GeometryKind;

#[derive(Debug)]
pub enum Error {
    InvalidGeometryKind {
        expecting: GeometryKind,
        got: GeometryKind,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl Error {
    pub fn invalid_geometry_kind(expecting: GeometryKind, got: GeometryKind) -> Self {
        Self::InvalidGeometryKind { expecting, got }
    }
}

