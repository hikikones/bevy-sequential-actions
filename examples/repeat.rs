use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, playground::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlaygroundPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
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
