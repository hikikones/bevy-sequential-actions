use bevy::prelude::*;
use bevy_sequential_actions::ActionsBundle;

use super::assets::*;

#[derive(Component)]
pub struct Actor;

pub trait SpawnActorExt {
    fn spawn_actor(&mut self, translation: Vec3, rotation: Quat) -> Entity;
}

impl SpawnActorExt for Commands<'_, '_> {
    fn spawn_actor(&mut self, translation: Vec3, rotation: Quat) -> Entity {
        self.spawn()
            .insert_bundle(SpatialBundle::from_transform(Transform {
                translation,
                rotation,
                ..Default::default()
            }))
            .insert_bundle(ActionsBundle::default())
            .insert(Actor)
            .id()
    }
}

pub(super) fn load_actor(
    actor_added_q: Query<Entity, Added<Actor>>,
    assets: Res<MyAssets>,
    mut commands: Commands,
) {
    for actor in actor_added_q.iter() {
        commands.entity(actor).with_children(|child| {
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
            let eye_scale = Vec3::splat(0.15);

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
}
