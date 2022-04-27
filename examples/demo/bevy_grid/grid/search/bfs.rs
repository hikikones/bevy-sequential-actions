use std::collections::VecDeque;

use bevy::utils::HashSet;

use crate::bevy_grid::*;

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
    pub fn flood_fill(&self, start: T::Cell, max: usize) -> Vec<T::Cell> {
        let mut queue: VecDeque<(T::Cell, usize)> = VecDeque::default();
        let mut visited: HashSet<T::Cell> = HashSet::default();

        queue.push_back((start, max));
        visited.insert(start);

        while let Some((cell, length)) = queue.pop_front() {
            if length == 0 {
                continue;
            }

            let tile = self.grid.get_tile(cell);
            for neighbor_cell in tile.neighbors(cell) {
                if let Some(neighbor) = self.grid.try_get_tile(neighbor_cell) {
                    if !neighbor.is_walkable() {
                        continue;
                    }

                    if !is_connected(cell, neighbor, neighbor_cell) {
                        continue;
                    }

                    if !visited.contains(&neighbor_cell) {
                        visited.insert(neighbor_cell);
                        queue.push_back((neighbor_cell, length - 1));
                    }
                }
            }
        }

        Vec::from_iter(visited.into_iter())
    }
}
