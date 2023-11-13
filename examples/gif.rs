use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Window {
                title: "secret".into(),
                resolution: bevy::window::WindowResolution::new(720.0, 480.0),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..Default::default()
            }
            .into(),
            ..Default::default()
        }),))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Agent
    commands
        .spawn((PbrBundle {
            mesh: meshes.add(shape::Capsule::default().into()),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ..Default::default()
        },))
        .with_children(|builder| {
            let icosphere = meshes.add(shape::Icosphere::default().try_into().unwrap());
            let black = materials.add(Color::BLACK.into());

            let eye_left = Vec3::new(-0.2, 0.6, -0.4);
            let eye_right = Vec3::new(-eye_left.x, eye_left.y, eye_left.z);
            let eye_scale = Vec3::splat(0.15);

            builder.spawn(PbrBundle {
                mesh: icosphere.clone(),
                material: black.clone(),
                transform: Transform {
                    translation: eye_left,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
            builder.spawn(PbrBundle {
                mesh: icosphere,
                material: black,
                transform: Transform {
                    translation: eye_right,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
        });

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::Z * 5.0),
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
