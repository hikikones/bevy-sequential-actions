#[derive(Clone, Copy)]
pub struct GridSize {
    columns: usize,
    rows: usize,
    floors: usize,
}

impl GridSize {
    pub fn new(columns: usize, rows: usize, floors: usize) -> Self {
        Self {
            columns,
            rows,
            floors,
        }
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn floors(&self) -> usize {
        self.floors
    }

    pub fn capacity(&self) -> usize {
        self.rows * self.columns * self.floors
    }
}
