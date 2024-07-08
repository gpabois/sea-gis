use super::{GeometryImpl, VectorMatrix, MBR};

pub type MultiLineStringCoordinates<const N: usize, U> = VectorMatrix<N, U>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Un ensemble de lignes brisées.
pub struct MultiLineString<const N: usize, U> {
    pub coordinates: MultiLineStringCoordinates<N, U>,
    pub srid: Option<u32>,
}

impl<const N: usize, U>  GeometryImpl for MultiLineString<N, U> {
    type Coordinates = MultiLineStringCoordinates<N, U>;

    fn new<C: Into<Self::Coordinates>>(coordinates: C) -> Self {
        Self {
            coordinates: coordinates.into(),
            srid: None,
        }
    }
}

impl<const N: usize, U> MultiLineString<N, U>
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
