use crate::bevy_grid::*;

pub trait GridTile: Default {
    type Cell: GridCell;
    type Neighbors: Iterator<Item = Self::Cell>;

    fn is_walkable(&self) -> bool;
    fn neighbors(&self, cell: Self::Cell) -> Self::Neighbors;

    // Default impls
    fn edge_weight(&self, _cell: Self::Cell, _neighbor: Self::Cell, _grid: &Grid<Self>) -> usize {
        1
    }

    fn heuristic(&self, cell: Self::Cell, goal: Self::Cell) -> usize {
        cell.distance(goal)
    }

    fn move_cost(&self) -> usize {
        1
    }
}
