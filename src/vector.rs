use num_traits::{CheckedNeg, FromPrimitive, Num, NumAssignOps, NumOps, ToPrimitive};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub};

use bracket_lib::prelude::Point;

use components::direction::Direction;
use components::position::Position;

#[derive(Debug, Default, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Vector<T: Num + Copy>(pub T, pub T);

impl<T: Num + ToPrimitive + Copy> Vector<T> {
    pub fn distance(&self, vector: Vector<T>) -> u32 {
        let a = self.0 - vector.0;
        let b = self.1 - vector.1;

        f32::sqrt((a * a + b * b).to_f32().unwrap()).floor() as u32
    }
}

impl<T: Num + Copy + CheckedNeg> From<Direction> for Vector<T> {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => Vector(T::zero(), T::one().checked_neg().unwrap()),
            Direction::East => Vector(T::one(), T::zero()),
            Direction::South => Vector(T::zero(), T::one()),
            Direction::West => Vector(T::one().checked_neg().unwrap(), T::zero()),
            Direction::NorthWest => Vector(
                T::one().checked_neg().unwrap(),
                T::one().checked_neg().unwrap(),
            ),
            Direction::NorthEast => Vector(T::one(), T::one().checked_neg().unwrap()),
            Direction::SouthWest => Vector(T::one(), T::one()),
            Direction::SouthEast => Vector(T::one().checked_neg().unwrap(), T::one()),
        }
    }
}

impl From<Position> for Vector<i32> {
    fn from(position: Position) -> Self {
        position.vector
    }
}

impl<T: Num + Copy> From<(T, T)> for Vector<T> {
    fn from(tuple: (T, T)) -> Self {
        Vector(tuple.0, tuple.1)
    }
}

impl<T: Num + FromPrimitive + Copy> From<Point> for Vector<T> {
    fn from(point: Point) -> Self {
        Vector(
            T::from_i32(point.x).unwrap(),
            T::from_i32(point.y.into()).unwrap(),
        )
    }
}

impl<T: Num + ToPrimitive + Copy> Into<Point> for Vector<T> {
    fn into(self) -> Point {
        Point {
            x: self.0.to_i32().unwrap(),
            y: self.1.to_i32().unwrap(),
        }
    }
}

impl<T: Add<Output = T> + Num + Copy> Add for Vector<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Sub<Output = T> + Num + Copy> Sub for Vector<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: Num + NumAssignOps + Copy> AddAssign for Vector<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<T: Num + NumOps + Copy> Mul<T> for Vector<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl<T: Num + NumAssignOps + Copy> MulAssign<T> for Vector<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
        self.1 *= rhs;
    }
}
