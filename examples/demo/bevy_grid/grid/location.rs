use crate::bevy_grid::*;

pub enum GridLocation {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl GridLocation {
    pub(crate) fn into_cell<T: GridTile>(&self, size: GridSize, floor: CellInt) -> T::Cell {
        match self {
            GridLocation::TopLeft => T::Cell::new(0, 0, floor),
            GridLocation::TopRight => T::Cell::new((size.columns() - 1) as CellInt, 0, floor),
            GridLocation::BottomLeft => T::Cell::new(0, (size.rows() - 1) as CellInt, floor),
            GridLocation::BottomRight => T::Cell::new(
                (size.columns() - 1) as CellInt,
                (size.rows() - 1) as CellInt,
                floor,
            ),
            GridLocation::Center => T::Cell::new(
                (size.columns() / 2) as CellInt,
                (size.rows() / 2) as CellInt,
                floor,
            ),
        }
    }
}
