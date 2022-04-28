use bevy::math::Vec3;

use crate::bevy_grid::*;

pub struct GridSquareIter<'a, T: GridTile> {
    grid: &'a Grid<T>,
    columns: CellInt,
    rows: CellInt,
    start: T::Cell,
    col: CellInt,
    row: CellInt,
}

impl<'a, T: GridTile> GridSquareIter<'a, T> {
    pub fn new(grid: &'a Grid<T>, point: CellPoint, radius: f32) -> Self {
        let top_left_pos = point + Vec3::new(-radius, 0.0, -radius);
        let bot_right_pos = point + Vec3::new(radius, 0.0, radius);

        let top_left_cell = grid.get_cell_clamped(top_left_pos);
        let bot_right_cell = grid.get_cell_clamped(bot_right_pos);

        let columns = bot_right_cell.column() - top_left_cell.column();
        let rows = bot_right_cell.row() - top_left_cell.row();

        Self {
            grid,
            columns,
            rows,
            start: top_left_cell,
            col: 0,
            row: 0,
        }
    }
}

impl<'a, T: GridTile> Iterator for GridSquareIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row > self.rows {
            return None;
        }

        let next = self.start + T::Cell::new(self.col, self.row, 0);
        let tile = self.grid.get_tile(next);

        self.col += 1;
        if self.col > self.columns {
            self.row += 1;
            self.col = 0;
        }

        Some(tile)
    }
}
