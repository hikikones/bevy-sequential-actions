use bevy::prelude::*;

use crate::bevy_grid::{GridCell, GridTile, SquareCell};

pub(super) struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tile {
    None,
    Ground,
    Event,
    Trap,
}

impl GridTile for Tile {
    type Cell = SquareCell;
    type Neighbors = std::array::IntoIter<Self::Cell, 8>;

    fn is_walkable(&self) -> bool {
        match self {
            Tile::None => false,
            _ => true,
        }
    }

    fn neighbors(&self, cell: SquareCell) -> Self::Neighbors {
        cell.neighbors()
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Ground
    }
}
