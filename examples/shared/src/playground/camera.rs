use bevy::{math::const_vec3, prelude::*};

pub const CAMERA_OFFSET: Vec3 = const_vec3!([0.0, 10.0, 8.0]);

#[derive(Component)]
pub struct CameraMain;

#[derive(Component)]
pub struct CameraPivot;

pub(super) fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(TransformBundle::default())
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
