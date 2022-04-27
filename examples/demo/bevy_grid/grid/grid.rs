use crate::bevy_grid::*;

pub struct Grid<T: GridTile> {
    size: GridSize,
    cell_size: CellSize,
    tiles: Vec<T>,
}

impl<T: GridTile> Grid<T> {
    pub fn new(size: GridSize, cell_size: CellSize) -> Self {
        let mut array: Vec<T> = Vec::with_capacity(size.capacity());
        for _ in 0..array.capacity() {
            array.push(T::default());
        }

        Self {
            size,
            cell_size,
            tiles: array,
        }
    }

    pub fn size(&self) -> GridSize {
        self.size
    }

    pub fn cell_size(&self) -> CellSize {
        self.cell_size
    }

    pub fn get_cell(&self, point: CellPoint) -> T::Cell {
        T::Cell::from_point(point, self.cell_size)
    }

    pub fn get_cell_at(&self, location: GridLocation, floor: CellInt) -> T::Cell {
        location.into_cell::<T>(self.size, floor)
    }

    pub fn get_cell_clamped(&self, point: CellPoint) -> T::Cell {
        let cell = T::Cell::from_point(point, self.cell_size);
        let col = cell.column().clamp(0, self.size.columns() as CellInt - 1);
        let row = cell.row().clamp(0, self.size.rows() as CellInt - 1);
        let floor = cell.floor().clamp(0, self.size.floors() as CellInt - 1);
        T::Cell::new(col, row, floor)
    }

    pub fn get_cell_from_index(&self, index: usize) -> T::Cell {
        let col = index % self.size.columns();
        let row = (index / self.size.columns()) % self.size.rows();
        let floor = index / (self.size.columns() * self.size.rows());
        T::Cell::new(col as CellInt, row as CellInt, floor as CellInt)
    }

    pub fn get_index_from_cell(&self, cell: T::Cell) -> usize {
        self.size.columns() * self.size.rows() * cell.floor() as usize
            + self.size.columns() * cell.row() as usize
            + cell.column() as usize
    }

    pub fn try_get_cell(&self, point: CellPoint) -> Option<T::Cell> {
        let cell = T::Cell::from_point(point, self.cell_size);
        if self.is_cell_outside(cell) {
            return None;
        }
        Some(cell)
    }

    pub fn is_cell_outside(&self, cell: T::Cell) -> bool {
        cell.column() < 0
            || cell.row() < 0
            || cell.floor() < 0
            || cell.column() >= self.size.columns() as CellInt
            || cell.row() >= self.size.rows() as CellInt
            || cell.floor() >= self.size.floors() as CellInt
    }

    pub fn get_tile(&self, cell: T::Cell) -> &T {
        let index = self.get_index_from_cell(cell);
        &self.tiles[index]
    }

    pub fn get_tile_mut(&mut self, cell: T::Cell) -> &mut T {
        let index = self.get_index_from_cell(cell);
        &mut self.tiles[index]
    }

    pub fn try_get_tile(&self, cell: T::Cell) -> Option<&T> {
        if self.is_cell_outside(cell) {
            return None;
        }
        Some(self.get_tile(cell))
    }

    pub fn set_tile(&mut self, cell: T::Cell, tile: T) {
        let index = self.get_index_from_cell(cell);
        self.tiles[index] = tile;
    }
}

impl<'a, T: GridTile> IntoIterator for &'a Grid<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.tiles.iter()
    }
}

impl<'a, T: GridTile> IntoIterator for &'a mut Grid<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.tiles.iter_mut()
    }
}
