use bevy::prelude::*;

mod agent;
mod assets;
mod camera;
mod level;

pub use agent::*;
pub use camera::*;
pub use level::*;

pub struct BootstrapPlugin;

impl Plugin for BootstrapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(assets::AssetsPlugin)
            .add_plugin(level::LevelPlugin)
            .add_plugin(camera::CameraPlugin)
            .add_plugin(agent::AgentPlugin)
            .add_system(bevy::window::close_on_esc);
    }
}
