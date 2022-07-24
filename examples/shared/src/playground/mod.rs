use bevy::prelude::*;

mod assets;
mod camera;
mod level;
mod player;

pub struct PlaygroundPlugin;

impl Plugin for PlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(assets::AssetsPlugin)
            .add_plugin(level::LevelPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_plugin(camera::CameraPlugin);
    }
}
