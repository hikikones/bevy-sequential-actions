use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        // Run input before update to avoid modifying actions in the same frame
        .add_system_to_stage(CoreStage::PreUpdate, input)
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
    mut is_stopped: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let player = player_q.single();

        if *is_stopped {
            commands.actions(player).next();
        } else {
            commands.actions(player).stop();
        }

        *is_stopped = !*is_stopped;
    }
}
