#[derive(Debug, Clone, PartialEq, Eq)]
/// Rectangle à limite minimum (minimum bounding rectangle)
pub struct MBR<U> {
    pub min_x: U,
    pub min_y: U,
    pub max_x: U,
    pub max_y: U,
}
