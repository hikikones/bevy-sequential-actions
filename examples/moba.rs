use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(input_move)
                .with_system(input_clear),
        )
        .run();
}

fn setup(mut commands: Commands, mut ground_q: Query<&mut Transform, With<Ground>>) {
    commands.spawn_agent(AgentConfig::default());
    ground_q.single_mut().scale = Vec3::new(100.0, 1.0, 100.0);
}

fn input_move(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    camera_q: Query<(&Camera, &GlobalTransform), With<CameraMain>>,
    agent_q: Query<Entity, With<Agent>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Right) {
        let window = windows.get_primary().unwrap();
        if let Some(cursor_pos) = window.cursor_position() {
            let (camera, transform) = camera_q.single();
            if let Some(ray) = camera.viewport_to_world(transform, cursor_pos) {
                let agent = agent_q.single();
                let hit = ray.intersect_plane();
                let mut actions = commands.actions(agent);

                if !keyboard.pressed(KeyCode::LShift) {
                    actions.clear();
                }

                actions.add(MoveAction::new(MoveConfig {
                    target: hit,
                    speed: 6.0,
                    rotate: true,
                }));
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

trait IntersectPlane {
    fn intersect_plane(&self) -> Vec3;
}

impl IntersectPlane for Ray {
    fn intersect_plane(&self) -> Vec3 {
        let plane_normal = Vec3::Y;
        let denominator = self.direction.dot(plane_normal);
        let intersect_dist = plane_normal.dot(-self.origin) / denominator;
        self.direction * intersect_dist + self.origin
    }
}
