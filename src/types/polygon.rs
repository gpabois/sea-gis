use super::{VectorMatrix, MBR};

pub type PolygonCoordinates<const N: usize, U> = VectorMatrix<N, U>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Un polygone
pub struct Polygon<const N: usize, U> {
    pub coordinates: PolygonCoordinates<N, U>,
    pub srid: u32,
}

impl<const N: usize, U> Polygon<N, U>
where
    U: PartialEq + Clone,
{
    /// Crée un nouveau polygone, avec un SRID de 4326 par défaut.
    pub fn new<V: Into<PolygonCoordinates<N, U>>>(args: V) -> Self {
        let mut coordinates: PolygonCoordinates<N, U> = args.into();

        // Ensure we close all rings.
        coordinates.iter_mut().for_each(|v| v.close_ring());

        Self {
            coordinates,
            srid: 4326,
        }
    }
}

impl<const N: usize, U> Polygon<N, U>
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

