use super::{VectorArray, MBR};

pub type MultiPointCoordinates<const N: usize, U> = VectorArray<N, U>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Un ensemble de point non relié dans un espace 2D.
pub struct MultiPoint<const N: usize, U> {
    pub coordinates: MultiPointCoordinates<N, U>,
    pub srid: u32,
}

impl<const N: usize, U> MultiPoint<N, U>
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

impl<const N: usize, U> MultiPoint<N, U> {
    pub fn new<V: Into<MultiPointCoordinates<N, U>>>(coordinates: V) -> Self {
        Self {
            coordinates: coordinates.into(),
            srid: 4326,
        }
    }
}