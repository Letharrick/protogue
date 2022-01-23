use bracket_lib::prelude::RandomNumberGenerator;
use vector::Vector;

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    North,
    East,
    South,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl Direction {
    pub fn as_unit_vector(&self) -> Vector<i32> {
        (*self).into()
    }

    pub fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
            Direction::NorthEast => Direction::SouthWest,
            Direction::SouthWest => Direction::NorthEast,
            Direction::SouthEast => Direction::NorthWest,
        }
    }

    pub fn random(direction: Option<Direction>, magnitude: Option<u32>) -> Self {
        let mut rng = RandomNumberGenerator::new();
        let _magnitude = magnitude.unwrap_or(1);

        direction.unwrap_or(match rng.range(0, 8) {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            4 => Direction::NorthWest,
            5 => Direction::NorthEast,
            6 => Direction::SouthEast,
            7 => Direction::SouthWest,
            _ => direction.unwrap(),
        })
    }

    pub fn all() -> [Direction; 8] {
        [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
            Direction::NorthWest,
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::SouthWest,
        ]
    }
}
