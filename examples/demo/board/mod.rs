use bevy::prelude::*;

mod highlight;
mod tile;

pub use crate::bevy_grid::*;
pub use highlight::*;
pub use tile::*;

use crate::assets::*;

pub(super) struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::default())
            .add_plugin(TilePlugin)
            .add_plugin(HighlightPlugin)
            .add_startup_system(spawn_board)
            .add_startup_system(spawn_light);
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

        let mut grid: Grid<Tile> = Grid::new(grid_size, cell_size);

        grid.set_tile(SquareCell::new(2, 2, 0), Tile::Blocked);
        grid.set_tile(SquareCell::new(6, 2, 0), Tile::Blocked);
        grid.set_tile(SquareCell::new(2, 6, 0), Tile::Blocked);
        grid.set_tile(SquareCell::new(6, 6, 0), Tile::Blocked);

        grid.set_tile(SquareCell::new(4, 2, 0), Tile::Event);
        grid.set_tile(SquareCell::new(2, 4, 0), Tile::Trap);
        grid.set_tile(SquareCell::new(6, 4, 0), Tile::Trap);
        grid.set_tile(SquareCell::new(4, 6, 0), Tile::Event);

        Self(grid)
    }
}

fn spawn_board(
    board: Res<Board>,
    meshes: Res<Meshes>,
    materials: Res<Materials>,
    mut commands: Commands,
) {
    let height = 1.0;
    let size = board.cell_size();
    let scale = Vec3::new(size, height, size);

    for i in 0..board.size().capacity() {
        let cell = board.get_cell_from_index(i);
        let tile = board.get_tile(cell);
        let translation = cell.as_point(size) + Vec3::new(0.0, -height * 0.5, 0.0);
        let material = match tile {
            Tile::Blocked => materials.get(MaterialName::None),
            Tile::Ground => materials.get(MaterialName::DarkGray),
            Tile::Event => materials.get(MaterialName::Gold),
            Tile::Trap => materials.get(MaterialName::Maroon),
        };

        commands.spawn_bundle(PbrBundle {
            mesh: meshes.get(MeshName::Cube),
            material,
            transform: Transform {
                translation,
                scale,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

fn spawn_light(mut commands: Commands) {
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 15000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
