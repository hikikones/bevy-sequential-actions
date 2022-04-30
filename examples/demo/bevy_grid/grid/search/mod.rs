mod astar;
mod bfs;
mod dijkstra;

pub use astar::*;
pub use bfs::*;
pub use dijkstra::*;

use crate::bevy_grid::{Grid, GridTile};

pub enum EdgeWeight {
    Const(usize),
    Single,
    Custom,
}

impl EdgeWeight {
    fn cost<T: GridTile>(&self, tile: &T, cell: T::Cell, other: T::Cell, grid: &Grid<T>) -> usize {
        match *self {
            EdgeWeight::Const(cost) => cost,
            EdgeWeight::Single => tile.edge_cost(),
            EdgeWeight::Custom => tile.edge_cost_custom(cell, other, grid),
        }
    }
}

fn is_connected<T: GridTile>(
    node_cell: T::Cell,
    neighbor_tile: &T,
    neighbor_cell: T::Cell,
) -> bool {
    for cell in neighbor_tile.neighbors(neighbor_cell) {
        if cell == node_cell {
            return true;
        }
    }
    false
}
