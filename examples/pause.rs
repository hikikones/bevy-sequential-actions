use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::FromVec3Ext};

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
    let seconds = Random::new(0.5, 2.0);
    let rotation = Random::new(Vec3::ZERO, Vec3::Y * std::f32::consts::TAU);
    let position = Random::new(Vec3::new(-7.0, 0.0, -4.0), Vec3::new(7.0, 0.0, 4.0));

    for _ in 0..10 {
        let actor = commands.spawn_actor(position.value(), Quat::from_vec3(rotation.value()));

        commands
            .actions(actor)
            .config(AddConfig {
                order: AddOrder::Back,
                start: true,
                repeat: Repeat::Forever,
            })
            .add(WaitAction::new(seconds))
            .add(RotateAction::new(rotation))
            .add(WaitAction::new(seconds))
            .add(MoveAction::new(position));
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
