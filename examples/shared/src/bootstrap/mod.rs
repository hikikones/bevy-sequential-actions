use bevy::prelude::*;

mod agent;
mod assets;
mod camera;
mod level;

pub use agent::*;
pub use camera::*;

pub struct BootstrapPlugin;

impl Plugin for BootstrapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(assets::AssetsPlugin)
            .add_plugin(level::LevelPlugin)
            .add_plugin(camera::CameraPlugin)
            .add_plugin(agent::AgentPlugin)
            .add_system_to_stage(CoreStage::PreUpdate, bevy::window::close_on_esc);
    }
}
