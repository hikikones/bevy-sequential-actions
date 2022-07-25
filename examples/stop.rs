use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, playground::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlaygroundPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .add_system(input)
        .run();
}

fn setup(player_q: Query<Entity, With<Player>>, mut commands: Commands) {
    let player = player_q.single();
    commands
        .action(player)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: true,
        })
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(-Vec3::X * 4.0))
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(Vec3::X * 4.0));
}

fn input(
    keyboard: Res<Input<KeyCode>>,
    player_q: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut is_stopped: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let player = player_q.single();

        if *is_stopped {
            commands.action(player).next();
        } else {
            commands.action(player).stop();
        }

        *is_stopped = !*is_stopped;
    }
}
