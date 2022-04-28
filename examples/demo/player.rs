use bevy::prelude::*;

use crate::{assets::*, board::*};
use bevy_sequential_actions::ActionsBundle;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player);
    }
}

#[derive(Component)]
pub struct Player;

fn spawn_player(
    board: Res<Board>,
    meshes: Res<Meshes>,
    materials: Res<Materials>,
    mut commands: Commands,
) {
    let center = board.get_cell_at(GridLocation::Center, 0);
    let pos = center.as_point(board.cell_size());

    commands
        .spawn()
        .insert(GlobalTransform::identity())
        .insert(Transform {
            translation: pos,
            ..Default::default()
        })
        .insert_bundle(ActionsBundle::default())
        .insert(Player)
        .with_children(|child| {
            // Capsule
            child.spawn_bundle(PbrBundle {
                mesh: meshes.get(MeshName::Capsule),
                material: materials.get(MaterialName::White),
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
                mesh: meshes.get(MeshName::Icosphere),
                material: materials.get(MaterialName::Black),
                transform: Transform {
                    translation: eye_left,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
            child.spawn_bundle(PbrBundle {
                mesh: meshes.get(MeshName::Icosphere),
                material: materials.get(MaterialName::Black),
                transform: Transform {
                    translation: eye_right,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
        });
}
