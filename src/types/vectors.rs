use std::ops::{Deref, DerefMut};

/// Un vecteur dimension N.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Vector<const N: usize, U>([U; N]);

impl<const N: usize, U> From<[U; N]> for Vector<N, U> {
    fn from(value: [U; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize, U> Vector<N, U> {
    pub fn new(coordinates: [U; N]) -> Self {
        Self(coordinates)
    }
}

impl<const N: usize, U> Vector<N, U>
where
    U: Copy,
{
    pub fn x(&self) -> U {
        self.0[0]
    }

    pub fn y(&self) -> U {
        self.0[1]
    }

    pub fn z(&self) -> U {
        self.0[2]
    }
}

impl<const N: usize, U> IntoIterator for Vector<N, U> {
    type Item = U;
    type IntoIter = std::array::IntoIter<U, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize, U> Deref for Vector<N, U> {
    type Target = [U; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, U> DerefMut for Vector<N, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Un tableau 1D de vecteur de dimension N.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorArray<const N: usize, U>(Vec<Vector<N, U>>);

impl<const N: usize, U> VectorArray<N, U> {
    pub fn new(a: Vec<Vector<N, U>>) -> Self {
        Self(a)
    }
}

impl<const R: usize, const N: usize, U> From<[[U; N]; R]> for VectorArray<N, U> {
    fn from(value: [[U; N]; R]) -> Self {
        Self::from_iter(value.into_iter().map(Vector::from))
    }
}

impl<const N: usize, U> VectorArray<N, U>
where
    U: Clone + PartialEq,
{
    /// Vérifie que la liste de points forme un anneau, sinon le ferme automatiquement.
    pub fn close_ring(&mut self) {
        if self.first() != self.last() {
            self.0
                .push(self.first().cloned().expect("ring must not be empty"));
        }
    }
}

impl<const N: usize, U> VectorArray<N, U>
where
    U: Copy + PartialOrd,
{
    pub fn min_x(&self) -> U {
        self.0
            .iter()
            .map(Vector::x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn max_x(&self) -> U {
        self.0
            .iter()
            .map(Vector::x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn min_y(&self) -> U {
        self.0
            .iter()
            .map(Vector::y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn max_y(&self) -> U {
        self.0
            .iter()
            .map(Vector::y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}

impl<const N: usize, U> FromIterator<Vector<N, U>> for VectorArray<N, U> {
    fn from_iter<T: IntoIterator<Item = Vector<N, U>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize, U> FromIterator<[U;N]> for VectorArray<N, U> {
    fn from_iter<T: IntoIterator<Item = [U;N]>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(Vector::from))
    }
}

impl<const N: usize, U> IntoIterator for VectorArray<N, U> {
    type Item = Vector<N, U>;
    type IntoIter = <Vec<Vector<N, U>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize, U> Deref for VectorArray<N, U> {
    type Target = [Vector<N, U>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
/// Une matrice 2D de vecteur de dimension N.
pub struct VectorMatrix<const N: usize, U>(Vec<VectorArray<N, U>>);

impl<const N: usize, U> VectorMatrix<N, U> {
    pub fn new(coordinates: Vec<VectorArray<N, U>>) -> Self {
        Self(coordinates)
    }
}

impl<const N: usize, U, T1> From<T1> for VectorMatrix<N, U>
where
    VectorArray<N, U>: From<T1>,
{
    fn from(value: T1) -> Self {
        Self::new(vec![VectorArray::from(value)])
    }
}

impl<const N: usize, U, T1, T2> From<(T1, T2)> for VectorMatrix<N, U>
where
    VectorArray<N, U>: From<T1>,
    VectorArray<N, U>: From<T2>,
{
    fn from(value: (T1, T2)) -> Self {
        Self::new(vec![VectorArray::from(value.0), VectorArray::from(value.1)])
    }
}

impl<const N: usize, U, T1, T2, T3> From<(T1, T2, T3)> for VectorMatrix<N, U>
where
    VectorArray<N, U>: From<T1>,
    VectorArray<N, U>: From<T2>,
    VectorArray<N, U>: From<T3>,
{
    fn from(value: (T1, T2, T3)) -> Self {
        Self::new(vec![
            VectorArray::from(value.0),
            VectorArray::from(value.1),
            VectorArray::from(value.2),
        ])
    }
}

impl<const N: usize, U, T1, T2, T3, T4> From<(T1, T2, T3, T4)> for VectorMatrix<N, U>
where
    VectorArray<N, U>: From<T1>,
    VectorArray<N, U>: From<T2>,
    VectorArray<N, U>: From<T3>,
    VectorArray<N, U>: From<T4>,
{
    fn from(value: (T1, T2, T3, T4)) -> Self {
        Self::new(vec![
            VectorArray::from(value.0),
            VectorArray::from(value.1),
            VectorArray::from(value.2),
            VectorArray::from(value.3),
        ])
    }
}


impl<const N: usize, U> Deref for VectorMatrix<N, U> {
    type Target = [VectorArray<N, U>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, U> DerefMut for VectorMatrix<N, U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize, U> IntoIterator for VectorMatrix<N, U> {
    type Item = VectorArray<N, U>;
    type IntoIter = <Vec<VectorArray<N, U>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize, U> FromIterator<VectorArray<N,U>> for VectorMatrix<N, U> {
    fn from_iter<T: IntoIterator<Item = VectorArray<N,U>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize, U> FromIterator<Vec<[U;N]>> for VectorMatrix<N, U> {
    fn from_iter<T: IntoIterator<Item = Vec<[U;N]>>>(iter: T) -> Self {
        let arrays = iter
        .into_iter()
        .map(VectorArray::from_iter);

        Self::from_iter(arrays)
    }
}


impl<const N: usize, U> VectorMatrix<N, U>
where
    U: Copy + PartialOrd,
{
    pub fn min_x(&self) -> U {
        self.0
            .iter()
            .map(VectorArray::min_x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn max_x(&self) -> U {
        self.0
            .iter()
            .map(VectorArray::max_x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn min_y(&self) -> U {
        self.0
            .iter()
            .map(VectorArray::min_y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn max_y(&self) -> U {
        self.0
            .iter()
            .map(VectorArray::max_y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
/// Un tenseur 3D de vecteur de dimension N
pub struct VectorTensor<const N: usize, U>(Vec<VectorMatrix<N, U>>);

impl<const N: usize, U> VectorTensor<N, U> {
    pub fn new(coordinates: Vec<VectorMatrix<N, U>>) -> Self {
        Self(coordinates)
    }
}


impl<const N: usize, U> Deref for VectorTensor<N, U> {
    type Target = [VectorMatrix<N, U>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, U> IntoIterator for VectorTensor<N, U> {
    type Item = VectorMatrix<N, U>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const N: usize, U> FromIterator<VectorMatrix<N,U>> for VectorTensor<N, U> {
    fn from_iter<T: IntoIterator<Item = VectorMatrix<N,U>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize, U> FromIterator<Vec<Vec<[U;N]>>> for VectorTensor<N, U> {
    fn from_iter<T: IntoIterator<Item = Vec<Vec<[U;N]>>>>(iter: T) -> Self {
        let matrixes = iter
        .into_iter()
        .map(VectorMatrix::from_iter);

        Self::from_iter(matrixes)
    }
}

impl<const N: usize, U> VectorTensor<N, U>
where
    U: Copy + PartialOrd,
{
    pub fn min_x(&self) -> U {
        self.0
            .iter()
            .map(VectorMatrix::min_x)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn max_x(&self) -> U {
        self.0
            .iter()
            .map(VectorMatrix::max_x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn min_y(&self) -> U {
        self.0
            .iter()
            .map(VectorMatrix::min_y)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    pub fn max_y(&self) -> U {
        self.0
            .iter()
            .map(VectorMatrix::max_y)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}

impl<const N: usize, U, T1> From<T1> for VectorTensor<N, U>
where
    VectorMatrix<N, U>: From<T1>,
{
    fn from(value: T1) -> Self {
        Self::new(vec![VectorMatrix::from(value)])
    }
}
