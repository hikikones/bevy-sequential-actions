use std::hash::Hash;
use std::ops::Add;

use bevy::math::{IVec3, Vec3};

pub type CellInt = i32;
pub type CellSize = f32;
pub type CellPoint = Vec3;

pub trait GridCell
where
    Self: Default
        + Clone
        + Copy
        + PartialEq
        + Eq
        + Hash
        + Add<Self, Output = Self>
        + Add<IVec3, Output = Self>,
{
    type Neighbors: Iterator<Item = Self>;
    type Direction: Into<IVec3>;

    fn new(column: CellInt, row: CellInt, floor: CellInt) -> Self;

    fn column(&self) -> CellInt;
    fn row(&self) -> CellInt;
    fn floor(&self) -> CellInt;

    fn from_point(point: CellPoint, size: CellSize) -> Self;
    fn as_point(&self, size: CellSize) -> CellPoint;

    fn neighbors(&self) -> Self::Neighbors;

    // Default impls
    fn adjacent(&self, direction: Self::Direction) -> Self {
        let dir: IVec3 = direction.into();
        *self + dir
    }

    fn distance(&self, other: Self) -> usize {
        let x = other.column() - self.column();
        let y = other.row() - self.row();
        let z = other.floor() - self.floor();
        (x * x + y * y + z * z) as usize
    }
}
