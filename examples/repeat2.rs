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
    // Fetch player entity that already has ActionsBundle
    let player = player_q.single();

    // Add four actions with custom config that will repeat forever in the added order.
    commands
        .action(player)
        .config(AddConfig {
            order: AddOrder::Back, // Add each action to the back of the queue
            start: true,           // Start action if nothing is currently running
            repeat: true,          // Repeat each action when it has finished
        })
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(-Vec3::X * 4.0))
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(Vec3::X * 4.0));
}
