use super::{VectorTensor, MBR};

pub type MultiPolygonCoordinates<const N: usize, U> = VectorTensor<N, U>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Un ensemble de polygones
pub struct MultiPolygon<const N: usize, U> {
    pub coordinates: MultiPolygonCoordinates<N, U>,
    pub srid: u32,
}

impl<const N: usize, U> MultiPolygon<N, U> {
    pub fn new<V: Into<MultiPolygonCoordinates<N, U>>>(coordinates: V) -> Self {
        Self {
            coordinates: coordinates.into(),
            srid: 4326,
        }
    }

    pub fn new_with_srid(coordinates: MultiPolygonCoordinates<N, U>, srid: u32) -> Self {
        Self { coordinates, srid }
    }
}

impl<const N: usize, U> MultiPolygon<N, U>
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

