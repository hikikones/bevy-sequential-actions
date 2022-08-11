use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PreUpdate, input)
        .run();
}

fn setup(mut commands: Commands) {
    let actor = commands.spawn_actor(Vec3::ZERO, Quat::IDENTITY);

    commands
        .actions(actor)
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
    actor_q: Query<Entity, With<Actor>>,
    mut commands: Commands,
    mut is_paused: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let actor = actor_q.single();

        if *is_paused {
            commands.actions(actor).next();
        } else {
            commands.actions(actor).pause();
        }

        *is_paused = !*is_paused;
    }
}
