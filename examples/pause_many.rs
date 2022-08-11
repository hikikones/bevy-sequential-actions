use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::RandomExt};

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
    let min_wait = 0.5;
    let max_wait = 2.0;

    let min_move = Vec3::new(-7.0, 0.0, -4.0);
    let max_move = min_move * -1.0;

    let min_rot = Vec3::ZERO;
    let max_rot = Vec3::Y * std::f32::consts::PI * 2.0;

    for _ in 0..20 {
        let actor = commands.spawn_actor(
            Vec3::random(min_move, max_move),
            Quat::random(min_rot, max_rot),
        );

        commands
            .actions(actor)
            .config(AddConfig {
                order: AddOrder::Back,
                start: true,
                repeat: true,
            })
            .add(WaitRandomAction::new(min_wait, max_wait))
            .add(RotateRandomAction::new(min_rot, max_rot))
            .add(WaitRandomAction::new(min_wait, max_wait))
            .add(MoveRandomAction::new(min_move, max_move));
    }
}

fn input(
    keyboard: Res<Input<KeyCode>>,
    actors_q: Query<Entity, With<ActionMarker>>,
    mut commands: Commands,
    mut is_paused: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for actor in actors_q.iter() {
            if *is_paused {
                commands.actions(actor).next();
            } else {
                commands.actions(actor).pause();
            }
        }

        *is_paused = !*is_paused;
    }
}
