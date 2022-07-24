use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, playground::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlaygroundPlugin)
        .add_plugin(WaitActionPlugin)
        .add_plugin(MoveActionPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(player_q: Query<Entity, With<Player>>, mut commands: Commands) {
    let player = player_q.single();
    commands
        .action(player)
        .add(WaitAction::new(random_f32()))
        .add(MoveAction::new(random_vec3()))
        .add(WaitAction::new(random_f32()))
        .add(MoveAction::new(random_vec3()))
        .add(WaitAction::new(random_f32()))
        .add(MoveAction::new(random_vec3()))
        .add(WaitAction::new(random_f32()))
        .add(QuitAction::new());
}

fn random_f32() -> f32 {
    0.5 + fastrand::f32() * 2.0
}

fn random_vec3() -> Vec3 {
    let x = fastrand::isize(-5..=5) as f32 * fastrand::f32();
    let y = 0.0;
    let z = fastrand::isize(-5..=5) as f32 * fastrand::f32();
    Vec3::new(x, y, z)
}
