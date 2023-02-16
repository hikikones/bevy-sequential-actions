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
    let agent = commands.spawn_agent(AgentConfig::default());
    let camera_pivot = camera_q.single();

    commands
        .actions(agent)
        .repeat(Repeat::Forever)
        .add(WaitAction::new(1.0))
        .add_parallel(actions![
            MoveAction::new(MoveConfig {
                target: Vec3::X * 3.0,
                speed: Random::new(0.5, 5.0),
                rotate: false,
            }),
            RotateAction::new(RotateConfig {
                target: RotateType::Look(Vec3::X),
                speed: Random::new(std::f32::consts::FRAC_PI_8, std::f32::consts::PI),
            }),
            LerpAction::new(LerpConfig {
                target: camera_pivot,
                lerp_type: LerpType::Rotation(Quat::from_look(-Vec3::X, Vec3::Y)),
                duration: Random::new(2.0, 6.0),
            }),
            |agent: Entity, world: &mut World, _commands: &mut ActionCommands| {
                println!("on_start");
                world
                    .get_mut::<ActionFinished>(agent)
                    .unwrap()
                    .confirm_and_persist();
            }
        ])
        .add(WaitAction::new(1.0))
        .add_parallel(actions![
            MoveAction::new(MoveConfig {
                target: -Vec3::X * 3.0,
                speed: Random::new(0.5, 5.0),
                rotate: false,
            }),
            RotateAction::new(RotateConfig {
                target: RotateType::Look(-Vec3::X),
                speed: Random::new(std::f32::consts::FRAC_PI_8, std::f32::consts::PI),
            }),
            LerpAction::new(LerpConfig {
                target: camera_pivot,
                lerp_type: LerpType::Rotation(Quat::from_look(Vec3::X, Vec3::Y)),
                duration: Random::new(2.0, 6.0),
            }),
            (
                |agent: Entity, world: &mut World, _commands: &mut ActionCommands| {
                    println!("on_start and... ");
                    world
                        .get_mut::<ActionFinished>(agent)
                        .unwrap()
                        .confirm_and_persist();
                },
                |_agent: Entity, _world: &mut World, _reason: StopReason| {
                    println!("on_stop");
                }
            )
        ]);
}
