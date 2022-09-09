use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{
    actions::*,
    bootstrap::*,
    extensions::{LookRotationExt, RunSystemExt},
};

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
            // Repeat the action zero times, i.e. run only once
            repeat: Repeat::Amount(0),
        })
        .add(MoveAction::new(-Vec3::X * 2.0))
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(Vec3::X * 3.0))
        .add(WaitAction::new(1.0));

    // Add a list of actions
    commands
        .actions(actor)
        .config(AddConfig {
            // This time, add each action to the front of the queue
            order: AddOrder::Front,
            ..Default::default()
        })
        .add_many(
            ExecutionMode::Sequential,
            [
                WaitAction::new(10.0).into_boxed(),
                WaitAction::new(100.0).into_boxed(),
                WaitAction::new(1000.0).into_boxed(),
            ]
            .into_iter(),
        )
        // Ain't nobody got time to wait that long, so skip'em
        .skip()
        .skip()
        .skip();

    // Add a custom action that itself adds other actions
    commands.actions(actor).add(MyCustomAction);

    // Add an anonymous action using a closure
    commands
        .actions(actor)
        // Single closure for only the on_start method
        .add(
            |entity, _world: &mut World, commands: &mut ActionCommands| {
                // on_start
                commands.actions(entity).finish();
            },
        )
        // Tuple closure for both the on_start and on_stop methods
        .add((
            |entity, _world: &mut World, commands: &mut ActionCommands| {
                // on_start
                commands.actions(entity).finish();
            },
            |_entity, _world: &mut World, _reason| {
                // on_stop
            },
        ));

    // Get fancy...
    commands.actions(actor).add(FancyAction);

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

        let actions = [
            MoveAction::new(Vec3::ZERO).into_boxed(),
            WaitAction::new(1.0).into_boxed(),
            LerpAction::new(camera, LerpType::Position(CAMERA_OFFSET * 0.5), 1.0).into_boxed(),
            LerpAction::new(
                entity,
                LerpType::Rotation(Quat::look_rotation(Vec3::Z, Vec3::Y)),
                1.0,
            )
            .into_boxed(),
            WaitAction::new(1.0).into_boxed(),
            LerpAction::new(camera, LerpType::Position(CAMERA_OFFSET), 1.0).into_boxed(),
            WaitAction::new(0.5).into_boxed(),
            LerpAction::new(
                entity,
                LerpType::Rotation(Quat::look_rotation(-Vec3::Z, Vec3::Y)),
                0.5,
            )
            .into_boxed(),
        ];

        commands
            .actions(entity)
            .config(AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: Repeat::Amount(0),
            })
            .add_many(ExecutionMode::Sequential, actions.into_iter())
            .finish(); // TODO
    }

    fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
}

struct FancyAction;

impl Action for FancyAction {
    fn on_start(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // This action runs a system that adds another wait action.
        // When modifying actions using world inside the Action trait,
        // it is important that the modifications happens after the on_start method.

        commands
            .actions(entity)
            // Mutate the world after on_start has been called.
            .custom(|world| world.run_system(my_system))
            .finish();
    }

    fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
}

fn my_system(actor_q: Query<Entity, With<ActionMarker>>, mut commands: Commands) {
    let actor = actor_q.single();
    commands
        .actions(actor)
        .config(AddConfig {
            order: AddOrder::Front,
            ..Default::default()
        })
        .add(WaitAction::new(1.0));
}
