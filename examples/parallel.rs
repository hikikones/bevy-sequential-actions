use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::FromLookExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, camera_q: Query<Entity, With<CameraPivot>>) {
    let agent = commands.spawn_agent(Vec3::ZERO, Quat::IDENTITY);
    let camera_pivot = camera_q.single();

    commands
        .actions(agent)
        .config(AddConfig {
            order: AddOrder::Back,
            start: false,
            repeat: Repeat::Forever,
        })
        .add(WaitAction::new(1.0))
        .add_many(
            ExecutionMode::Parallel,
            [
                MoveAction::new(MoveConfig {
                    target: Vec3::X * 3.0,
                    speed: 4.0,
                    rotate: false,
                })
                .into_boxed(),
                RotateAction::new(RotateConfig {
                    target: RotateType::Look(Vec3::X),
                    speed: std::f32::consts::FRAC_PI_2,
                })
                .into_boxed(),
                LerpAction::new(LerpConfig {
                    target: camera_pivot,
                    lerp_type: LerpType::Rotation(Quat::from_look(-Vec3::X, Vec3::Y)),
                    duration: 2.0,
                })
                .into_boxed(),
                WaitAction::new(5.0).into_boxed(),
            ]
            .into_iter(),
        )
        .add(WaitAction::new(1.0))
        .add_many(
            ExecutionMode::Parallel,
            [
                MoveAction::new(MoveConfig {
                    target: -Vec3::X * 3.0,
                    speed: 1.0,
                    rotate: false,
                })
                .into_boxed(),
                RotateAction::new(RotateConfig {
                    target: RotateType::Look(-Vec3::X),
                    speed: std::f32::consts::FRAC_PI_2,
                })
                .into_boxed(),
                LerpAction::new(LerpConfig {
                    target: camera_pivot,
                    lerp_type: LerpType::Rotation(Quat::from_look(Vec3::X, Vec3::Y)),
                    duration: 3.0,
                })
                .into_boxed(),
                WaitAction::new(1.0).into_boxed(),
            ]
            .into_iter(),
        )
        .next();
}
