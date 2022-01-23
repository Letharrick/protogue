use components::direction::Direction;
use std::collections::LinkedList;
use vector::Vector;

pub struct Displacement {
    pub path: LinkedList<Vector<i32>>,
}

impl<C: Into<LinkedList<Vector<i32>>>> From<C> for Displacement {
    fn from(path: C) -> Self {
        Self { path: path.into() }
    }
}

impl From<Direction> for Displacement {
    fn from(direction: Direction) -> Self {
        let mut path = LinkedList::new();
        path.push_front(direction.into());

        Self { path }
    }
}
