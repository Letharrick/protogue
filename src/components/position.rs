use crate::vector::Vector;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Position {
    pub vector: Vector<i32>,
}

impl Position {
    pub fn new<V: Into<Vector<i32>>>(vector: V) -> Self {
        Self {
            vector: vector.into(),
        }
    }
}

impl From<Vector<i32>> for Position {
    fn from(vector: Vector<i32>) -> Self {
        Self { vector }
    }
}
