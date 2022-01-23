use vector::Vector;

#[derive(Default, Clone, Debug)]
pub struct Memory {
    pub(crate) spatial: Vec<Vector<i32>>, // Spatial memory (Previously seen locations
}
