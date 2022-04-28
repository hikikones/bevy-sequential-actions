use bevy::prelude::*;

mod assets;
mod bevy_actions;
mod bevy_extensions;
mod bevy_grid;
mod board;
mod camera;
mod player;

use assets::*;
use board::*;
use player::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Demo | Bevy Sequential Actions".into(),
            width: 1024.0,
            height: 720.0,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(bevy_actions::ActionsPlugin)
        .add_plugin(board::BoardPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
