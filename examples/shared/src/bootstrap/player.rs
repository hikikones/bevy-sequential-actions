use bevy::prelude::*;
use bevy_sequential_actions::ActionsBundle;

use super::assets::*;

#[derive(Component)]
pub struct Player;

pub(super) fn spawn_player(assets: Res<MyAssets>, mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(SpatialBundle::default())
        .insert_bundle(ActionsBundle::default())
        .insert(Player)
        .with_children(|child| {
            // Capsule
            child.spawn_bundle(PbrBundle {
                mesh: assets.get_mesh(MeshName::Capsule),
                material: assets.get_material(MaterialName::White),
                transform: Transform {
                    translation: Vec3::Y,
                    ..Default::default()
                },
                ..Default::default()
            });

            // Eyes
            let eye_left = Vec3::new(-0.2, 1.6, -0.4);
            let eye_right = Vec3::new(-eye_left.x, eye_left.y, eye_left.z);
            let eye_scale = Vec3::ONE * 0.15;

            child.spawn_bundle(PbrBundle {
                mesh: assets.get_mesh(MeshName::Icosphere),
                material: assets.get_material(MaterialName::Black),
                transform: Transform {
                    translation: eye_left,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
            child.spawn_bundle(PbrBundle {
                mesh: assets.get_mesh(MeshName::Icosphere),
                material: assets.get_material(MaterialName::Black),
                transform: Transform {
                    translation: eye_right,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}
