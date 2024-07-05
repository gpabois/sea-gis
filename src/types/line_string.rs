use super::{VectorArray, MBR};

pub type LineStringCoordinates<const N: usize, U> = VectorArray<N, U>;

/// Une suite de points reliés.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LineString<const N: usize, U> {
    pub coordinates: LineStringCoordinates<N, U>,
    pub srid: u32,
}


impl<const N: usize, U> LineString<N, U> {
    pub fn new<V: Into<LineStringCoordinates<N, U>>>(coordinates: V) -> Self {
        Self {
            coordinates: coordinates.into(),
            srid: 4326,
        }
    }
}

impl<const N: usize, U> LineString<N, U>
where
    U: Copy + PartialOrd,
{
    pub fn mbr(&self) -> MBR<U> {
        MBR {
            min_x: self.coordinates.min_x(),
            max_x: self.coordinates.max_x(),
            min_y: self.coordinates.min_y(),
            max_y: self.coordinates.max_y(),
        }
    }
}