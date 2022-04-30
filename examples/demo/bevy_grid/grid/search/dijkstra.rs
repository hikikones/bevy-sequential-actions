use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::utils::{HashMap, HashSet};

use crate::*;

use super::is_connected;

pub struct Dijkstra<'a, T: GridTile> {
    grid: &'a Grid<T>,
}

impl<'a, T: GridTile> Dijkstra<'a, T> {
    pub fn new(grid: &'a Grid<T>) -> Self {
        Self { grid }
    }

    pub fn fill(&self, start: T::Cell, max_cost: usize, edge_weight: EdgeWeight) -> Vec<T::Cell> {
        let mut heap: BinaryHeap<DijkstraNode<T::Cell>> = BinaryHeap::default();
        let mut cost: HashMap<T::Cell, usize> = HashMap::default();
        let mut visited: HashSet<T::Cell> = HashSet::default();

        heap.push(DijkstraNode::new(start, 0));
        cost.insert(start, 0);
        visited.insert(start);

        while let Some(node) = heap.pop() {
            let tile = self.grid.get_tile(node.cell);
            for neighbor_cell in tile.neighbors(node.cell) {
                if let Some(neighbor) = self.grid.try_get_tile(neighbor_cell) {
                    if !neighbor.is_walkable() {
                        continue;
                    }

                    if !is_connected(node.cell, neighbor, neighbor_cell) {
                        continue;
                    }

                    let edge_cost = edge_weight.cost(tile, node.cell, neighbor_cell, self.grid);
                    let accumulated_cost = cost[&node.cell] + edge_cost;

                    if accumulated_cost > max_cost {
                        continue;
                    }

                    if !cost.contains_key(&neighbor_cell) || accumulated_cost < cost[&neighbor_cell]
                    {
                        visited.insert(neighbor_cell);
                        cost.insert(neighbor_cell, accumulated_cost);
                        heap.push(DijkstraNode::new(neighbor_cell, accumulated_cost));
                    }
                }
            }
        }

        Vec::from_iter(visited.into_iter())
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct DijkstraNode<C: GridCell> {
    cell: C,
    cost: usize,
}

impl<C: GridCell> DijkstraNode<C> {
    fn new(cell: C, cost: usize) -> Self {
        Self { cell, cost }
    }
}

impl<C: GridCell> Ord for DijkstraNode<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl<C: GridCell> PartialOrd for DijkstraNode<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
