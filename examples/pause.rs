use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::FromEulerXYZExt};

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
        let agent = commands.spawn_agent(AgentConfig {
            position: position.value(),
            rotation: Quat::from_euler_xyz(rotation.value()),
        });

        commands
            .actions(agent)
            .repeat(Repeat::Forever)
            .add(WaitAction::new(seconds))
            .add(RotateAction::new(RotateConfig {
                target: RotateType::Euler(rotation),
                speed: Random::new(2.0, 4.0),
            }))
            .add(WaitAction::new(seconds))
            .add(MoveAction::new(MoveConfig {
                target: position,
                speed: Random::new(2.0, 5.0),
                rotate: true,
            }));
    }
}

fn input(
    keyboard: Res<Input<KeyCode>>,
    agents_q: Query<Entity, With<Agent>>,
    mut commands: Commands,
    mut is_paused: Local<bool>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for agent in agents_q.iter() {
            if *is_paused {
                commands.actions(agent).execute();
            } else {
                commands.actions(agent).pause();
            }
        }

        *is_paused = !*is_paused;
    }
}
