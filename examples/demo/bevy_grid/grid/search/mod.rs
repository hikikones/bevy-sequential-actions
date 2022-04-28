mod astar;
mod bfs;

pub use astar::*;
pub use bfs::*;

use crate::bevy_grid::GridTile;

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
