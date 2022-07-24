use bevy::prelude::*;

mod assets;
mod camera;
mod level;
mod player;

pub use camera::*;
pub use player::*;

pub struct PlaygroundPlugin;

impl Plugin for PlaygroundPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugin(assets::AssetsPlugin)
        //     .add_plugin(level::LevelPlugin)
        //     .add_plugin(player::PlayerPlugin)
        //     .add_plugin(camera::CameraPlugin)
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
            );
    }
}
