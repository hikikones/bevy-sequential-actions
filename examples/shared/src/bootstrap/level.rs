use bevy::prelude::*;

use super::assets::*;

pub(super) struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, spawn_level);
    }
}

fn spawn_level(assets: Res<MyAssets>, mut commands: Commands) {
    // Ground
    commands.spawn(PbrBundle {
        mesh: assets.mesh(MeshName::Cube),
        material: assets.material(MaterialName::DarkGray),
        transform: Transform {
            translation: -Vec3::Y * 0.5,
            scale: Vec3::new(16.0, 1.0, 10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 25000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
