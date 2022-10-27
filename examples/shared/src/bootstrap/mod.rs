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
        app.insert_resource(assets::MyAssets::default())
            .add_startup_system_set_to_stage(
                StartupStage::PreStartup,
                SystemSet::new()
                    .label("load_assets")
                    .with_system(assets::load),
            )
            .add_startup_system_set_to_stage(
                StartupStage::PreStartup,
                SystemSet::new()
                    .after("load_assets")
                    .with_system(level::spawn_level)
                    .with_system(camera::spawn_camera),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(agent::load_agent)
                    .with_system(bevy::window::close_on_esc),
            );
    }
}
