use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{actions::*, bootstrap::*, extensions::LookRotationExt};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BootstrapPlugin)
        .add_plugin(ActionsPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let actor = commands.spawn_actor(Vec3::ZERO, Quat::IDENTITY);

    // Add a single action with default config
    commands.actions(actor).add(WaitAction::new(1.0));

    // Add multiple actions with custom config
    commands
        .actions(actor)
        .config(AddConfig {
            // Add each action to the back of the queue
            order: AddOrder::Back,
            // Start action if nothing is currently running
            start: true,
            // Repeat the action
            repeat: false,
        })
        .add(MoveAction::new(-Vec3::X * 2.0))
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(Vec3::X * 3.0))
        .add(WaitAction::new(1.0));

    // Build a list of actions
    commands
        .actions(actor)
        .builder()
        .config(AddConfig {
            // This time, add each action to the front of the queue
            order: AddOrder::Front,
            ..Default::default()
        })
        .push(WaitAction::new(10.0))
        .push(WaitAction::new(100.0))
        .push(WaitAction::new(1000.0))
        // Since we are adding to the front, reverse list to get increasing wait times
        .reverse()
        // Submit the list of actions
        .submit()
        // Since we don't really wanna wait that long, let's skip the next three actions
        .skip()
        .skip()
        .skip();

    // Add a custom action that itself adds other actions
    commands.actions(actor).add(MyCustomAction);

    // Finally, quit the app
    commands.actions(actor).add(QuitAction);
}

struct MyCustomAction;

impl Action for MyCustomAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
        // This action adds a bunch of other actions to the front.
        // Every action must signal when they are done, so we call finish() at the end.

        let camera = world
            .query_filtered::<Entity, With<CameraMain>>()
            .single(world);

        commands
            .actions(entity)
            .builder()
            .config(AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: false,
            })
            .push(MoveAction::new(Vec3::ZERO))
            .push(WaitAction::new(1.0))
            .push(LerpAction::new(
                camera,
                LerpType::Position(CAMERA_OFFSET * 0.5),
                1.0,
            ))
            .push(LerpAction::new(
                entity,
                LerpType::Rotation(Quat::look_rotation(Vec3::Z, Vec3::Y)),
                1.5,
            ))
            .push(WaitAction::new(1.0))
            .push(LerpAction::new(
                camera,
                LerpType::Position(CAMERA_OFFSET),
                1.0,
            ))
            .push(WaitAction::new(1.0))
            .push(LerpAction::new(
                entity,
                LerpType::Rotation(Quat::look_rotation(-Vec3::Z, Vec3::Y)),
                1.0,
            ))
            .push(WaitAction::new(1.0))
            .reverse()
            .submit()
            .finish();
    }

    fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
}
