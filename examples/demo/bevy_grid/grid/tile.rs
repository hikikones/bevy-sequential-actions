use crate::{Grid, GridCell};

pub trait GridTile: Default {
    type Cell: GridCell;
    type Neighbors: Iterator<Item = Self::Cell>;

    fn is_walkable(&self) -> bool;
    fn neighbors(&self, cell: Self::Cell) -> Self::Neighbors;

    // Default impls
    fn edge_cost(&self) -> usize {
        1
    }

    fn edge_cost_custom(&self, _cell: Self::Cell, _other: Self::Cell, _grid: &Grid<Self>) -> usize {
        1
    }

    fn heuristic(&self, cell: Self::Cell, goal: Self::Cell) -> usize {
        cell.distance(goal)
    }
}
