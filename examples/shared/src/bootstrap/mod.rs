use bevy::prelude::*;

mod assets;
mod camera;
mod level;
mod player;

pub use camera::*;
pub use player::*;

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
                    .with_system(player::spawn_player)
                    .with_system(camera::spawn_camera),
            )
            .add_system(bevy::window::close_on_esc);
    }
}
