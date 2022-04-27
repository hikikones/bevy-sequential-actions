use std::ops::Add;

use bevy::math::{IVec2, IVec3, Vec3};

use crate::bevy_grid::*;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SquareCell {
    column: CellInt,
    row: CellInt,
}

impl GridCell for SquareCell {
    type Neighbors = std::array::IntoIter<Self, 8>;
    type Direction = SquareDirection;

    fn new(column: CellInt, row: CellInt, _floor: CellInt) -> Self {
        Self { column, row }
    }

    fn column(&self) -> CellInt {
        self.column
    }

    fn row(&self) -> CellInt {
        self.row
    }

    fn floor(&self) -> CellInt {
        0
    }

    fn from_point(point: CellPoint, size: CellSize) -> Self {
        Self {
            column: (point.x / size).floor() as CellInt,
            row: (point.z / size).floor() as CellInt,
        }
    }

    fn as_point(&self, size: CellSize) -> CellPoint {
        let x = self.column as CellSize * size;
        let z = self.row as CellSize * size;
        let point = CellPoint::new(x, 0.0, z);
        let size_half_offset = CellPoint::new(size * 0.5, 0.0, size * 0.5);
        point + size_half_offset
    }

    fn neighbors(&self) -> Self::Neighbors {
        let cell = *self;
        [
            cell + Self::new(-1, 1, 0),
            cell + Self::new(0, 1, 0),
            cell + Self::new(1, 1, 0),
            cell + Self::new(-1, 0, 0),
            cell + Self::new(1, 0, 0),
            cell + Self::new(-1, -1, 0),
            cell + Self::new(0, -1, 0),
            cell + Self::new(1, -1, 0),
        ]
        .into_iter()
    }
}

impl Add for SquareCell {
    type Output = Self;
    fn add(self, cell: Self) -> Self::Output {
        Self::new(self.column + cell.column, self.row + cell.row, 0)
    }
}

impl Add<IVec2> for SquareCell {
    type Output = Self;
    fn add(self, v: IVec2) -> Self::Output {
        Self::new(self.column + v.x, self.row + v.y, 0)
    }
}

impl Add<IVec3> for SquareCell {
    type Output = Self;
    fn add(self, v: IVec3) -> Self::Output {
        Self::new(self.column + v.x, self.row + v.z, 0)
    }
}

#[derive(Clone, Copy)]
pub enum SquareDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Into<IVec2> for SquareDirection {
    fn into(self) -> IVec2 {
        match self {
            SquareDirection::North => -IVec2::new(0, -1),
            SquareDirection::NorthEast => IVec2::new(1, -1),
            SquareDirection::East => IVec2::new(1, 0),
            SquareDirection::SouthEast => IVec2::new(1, 1),
            SquareDirection::South => IVec2::new(0, 1),
            SquareDirection::SouthWest => IVec2::new(-1, 1),
            SquareDirection::West => IVec2::new(-1, 0),
            SquareDirection::NorthWest => IVec2::new(-1, -1),
        }
    }
}

impl Into<IVec3> for SquareDirection {
    fn into(self) -> IVec3 {
        let v: IVec2 = self.into();
        IVec3::new(v.x, 0, v.y)
    }
}

impl Into<Vec3> for SquareDirection {
    fn into(self) -> Vec3 {
        let v: IVec2 = self.into();
        Vec3::new(v.x as f32, 0.0, v.y as f32)
    }
}
