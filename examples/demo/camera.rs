use bevy::{ecs::system::Command, math::const_vec3, prelude::*};

pub(super) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_camera).add_system(follow_target);
    }
}

pub const CAMERA_OFFSET: Vec3 = const_vec3!([0.0, 10.0, 8.0]);

#[derive(Component)]
pub struct CameraMain;

#[derive(Component)]
pub struct CameraPivot;

pub enum CameraCommand {
    Follow(Entity),
    Stop,
}

impl Command for CameraCommand {
    fn write(self, world: &mut World) {
        world.camera(self);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert(GlobalTransform::identity())
        .insert(Transform::identity())
        .insert(CameraPivot)
        .with_children(|child| {
            child
                .spawn_bundle(PerspectiveCameraBundle {
                    transform: Transform::from_translation(CAMERA_OFFSET)
                        .looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                })
                .insert(CameraMain);
        });
}

#[derive(Component)]
struct FollowTarget(Entity);

fn follow_target(
    mut camera_q: Query<(&mut Transform, &FollowTarget)>,
    target_q: Query<&Transform, Without<FollowTarget>>,
    time: Res<Time>,
) {
    for (mut transform, target) in camera_q.iter_mut() {
        if let Ok(target_transform) = target_q.get(target.0) {
            let distance = target_transform.translation.distance(transform.translation);
            let t = f32::powf(0.25, distance * time.delta_seconds());
            transform.translation =
                Vec3::lerp(target_transform.translation, transform.translation, t);
        }
    }
}

pub trait CameraCommandExt {
    fn camera(&mut self, command: CameraCommand);
}

impl CameraCommandExt for World {
    fn camera(&mut self, command: CameraCommand) {
        match command {
            CameraCommand::Follow(target) => {
                let pivot = self
                    .query_filtered::<Entity, With<CameraPivot>>()
                    .iter(self)
                    .next()
                    .unwrap();
                self.entity_mut(pivot).insert(FollowTarget(target));
            }
            CameraCommand::Stop => {
                let pivot = self
                    .query_filtered::<Entity, With<CameraPivot>>()
                    .iter(self)
                    .next()
                    .unwrap();
                self.entity_mut(pivot).remove::<FollowTarget>();
            }
        }
    }
}

impl CameraCommandExt for Commands<'_, '_> {
    fn camera(&mut self, command: CameraCommand) {
        self.add(command);
    }
}
