use std::collections::VecDeque;

use bevy::utils::HashSet;

use crate::*;

use super::is_connected;

pub struct Bfs<'a, T: GridTile> {
    grid: &'a Grid<T>,
}

impl<'a, T: GridTile> Bfs<'a, T> {
    pub fn new(grid: &'a Grid<T>) -> Self {
        Self { grid }
    }
}

impl<'a, T: GridTile> Bfs<'a, T> {
    pub fn fill(&self, start: T::Cell, max_cost: usize, edge_weight: EdgeWeight) -> Vec<T::Cell> {
        let mut queue: VecDeque<(T::Cell, usize)> = VecDeque::default();
        let mut visited: HashSet<T::Cell> = HashSet::default();

        queue.push_back((start, 0));
        visited.insert(start);

        while let Some((cell, current_cost)) = queue.pop_front() {
            let tile = self.grid.get_tile(cell);
            for neighbor_cell in tile.neighbors(cell) {
                if let Some(neighbor) = self.grid.try_get_tile(neighbor_cell) {
                    if !neighbor.is_walkable() {
                        continue;
                    }

                    if !is_connected(cell, neighbor, neighbor_cell) {
                        continue;
                    }

                    let edge_cost = edge_weight.cost(tile, cell, neighbor_cell, self.grid);
                    let next_cost = current_cost + edge_cost;

                    if next_cost > max_cost {
                        continue;
                    }

                    if !visited.contains(&neighbor_cell) {
                        visited.insert(neighbor_cell);
                        queue.push_back((neighbor_cell, next_cost));
                    }
                }
            }
        }

        Vec::from_iter(visited.into_iter())
    }
}
