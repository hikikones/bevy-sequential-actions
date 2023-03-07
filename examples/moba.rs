use bevy::{prelude::*, window::PrimaryWindow};
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .add_systems(
            (input_movement, input_clear)
                .in_base_set(CoreSet::PreUpdate)
                .after(bevy::input::InputSystem),
        )
        .run();
}

fn setup(mut commands: Commands, mut ground_q: Query<&mut Transform, With<Ground>>) {
    commands.spawn_agent(AgentConfig::default());
    ground_q.single_mut().scale = Vec3::new(100.0, 1.0, 100.0);
}

fn input_movement(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<CameraMain>>,
    agent_q: Query<Entity, With<Agent>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Right) {
        if let Some(cursor_pos) = window_q.single().cursor_position() {
            let (camera, transform) = camera_q.single();
            if let Some(ray) = camera.viewport_to_world(transform, cursor_pos) {
                if let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) {
                    let mut actions = commands.actions(agent_q.single());

                    if !keyboard.pressed(KeyCode::LShift) {
                        actions.clear();
                    }

                    actions.add(MoveAction::new(MoveConfig {
                        target: ray.direction * distance + ray.origin,
                        speed: 6.0,
                        rotate: true,
                    }));
                }
            }
        }
    }
}

fn input_clear(
    keyboard: Res<Input<KeyCode>>,
    agent_q: Query<Entity, With<Agent>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let agent = agent_q.single();
        commands.actions(agent).clear();
    }
}
