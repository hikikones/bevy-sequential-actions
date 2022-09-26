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

fn setup(mut commands: Commands, camera_q: Query<Entity, With<CameraMain>>) {
    let agent = commands.spawn_agent(Vec3::ZERO, Quat::IDENTITY);

    let camera = camera_q.single();

    commands
        .actions(agent)
        .add(WaitAction::new(1.0))
        .add_many(
            ExecutionMode::Parallel,
            [
                MoveAction::new(MoveConfig {
                    target: Vec3::X,
                    speed: 4.0,
                    rotate: false,
                })
                .into_boxed(),
                RotateAction::new(RotateConfig {
                    target: RotateType::Look(Vec3::X),
                    speed: 0.5,
                })
                .into_boxed(),
            ]
            .into_iter(),
        )
        .add(WaitAction::new(1.0))
        .add_many(
            ExecutionMode::Parallel,
            [
                MoveAction::new(MoveConfig {
                    target: -Vec3::X,
                    speed: 4.0,
                    rotate: false,
                })
                .into_boxed(),
                RotateAction::new(RotateConfig {
                    target: RotateType::Look(-Vec3::X),
                    speed: 0.5,
                })
                .into_boxed(),
            ]
            .into_iter(),
        );
}
