use bevy::prelude::*;

mod highlight;
mod tile;

pub use crate::bevy_grid::*;
pub use highlight::*;
pub use tile::*;

pub(super) struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::default())
            .add_plugin(TilePlugin)
            .add_plugin(HighlightPlugin);
    }
}

#[derive(Deref, DerefMut)]
pub struct Board(Grid<Tile>);

impl Board {
    pub fn _new(size: GridSize) -> Self {
        Self(Grid::new(size, 1.0))
    }
}

impl Default for Board {
    fn default() -> Self {
        let grid_size = GridSize::new(9, 9, 1);
        let cell_size: f32 = 1.0;

        let grid: Grid<Tile> = Grid::new(grid_size, cell_size);

        Self(grid)
    }
}
