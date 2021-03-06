use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(player_q: Query<Entity, With<Player>>, mut commands: Commands) {
    let player = player_q.single();

    let min_wait = 0.5;
    let max_wait = 2.0;

    let min_move = Vec3::new(-4.0, 0.0, -4.0);
    let max_move = min_move * -1.0;

    commands
        .actions(player)
        .config(AddConfig {
            order: AddOrder::Back,
            start: true,
            repeat: true,
        })
        .add(WaitRandomAction::new(min_wait, max_wait))
        .add(MoveRandomAction::new(min_move, max_move));
}
