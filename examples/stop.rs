use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .add_system(input)
        .run();
}

fn setup(player_q: Query<Entity, With<Player>>, mut commands: Commands) {
    let player = player_q.single();
    commands
        .actions(player)
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
    mut is_paused: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let player = player_q.single();

        // TODO: FIXME
        if *is_paused {
            commands.actions(player).next();
        } else {
            commands.actions(player).pause();
        }

        *is_paused = !*is_paused;
    }
}
