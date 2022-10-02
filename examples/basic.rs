use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::{
    actions::*,
    bootstrap::*,
    extensions::{FromLookExt, RunSystemExt},
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
    let agent = commands.spawn_agent(Vec3::ZERO, Quat::IDENTITY);

    // Add a single action with default config
    commands.actions(agent).add(WaitAction::new(1.0));

    // Add multiple actions with custom config
    commands
        .actions(agent)
        .config(AddConfig {
            // Add each action to the back of the queue
            order: AddOrder::Back,
            // Start the next action if nothing is currently running
            start: true,
            // Repeat the action zero times, i.e. run only once
            repeat: Repeat::Amount(0),
        })
        .add(MoveAction::new(MoveConfig {
            target: -Vec3::X * 2.0,
            speed: 4.0,
            rotate: true,
        }))
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(MoveConfig {
            target: Vec3::X * 3.0,
            speed: 6.0,
            rotate: true,
        }))
        .add(WaitAction::new(1.0));

    // Add a list of actions
    commands
        .actions(agent)
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
    commands.actions(agent).add(MyCustomAction);

    // Add an anonymous action using a closure
    commands
        .actions(agent)
        // Single closure for only the on_start method
        .add(|state: &mut WorldState, commands: &mut ActionCommands| {
            // on_start
            commands.actions(state.agent).next();
        })
        // Tuple closure for both the on_start and on_stop methods
        .add((
            |state: &mut WorldState, commands: &mut ActionCommands| {
                // on_start
                commands.actions(state.agent).next();
            },
            |_state: &mut WorldState, _reason| {
                // on_stop
            },
        ));

    // Get fancy...
    commands.actions(agent).add(FancyAction);

    // Finally, quit the app
    commands.actions(agent).add(QuitAction);
}

struct MyCustomAction;

impl Action for MyCustomAction {
    fn on_start(&mut self, state: &mut WorldState, commands: &mut ActionCommands) {
        // This action adds a bunch of other actions to the front.
        // Since this is all that it does, we call next() at the end.

        let camera = state
            .world
            .query_filtered::<Entity, With<CameraMain>>()
            .single(state.world);

        let actions = [
            MoveAction::new(MoveConfig {
                target: Vec3::ZERO,
                speed: 5.0,
                rotate: true,
            })
            .into_boxed(),
            WaitAction::new(1.0).into_boxed(),
            LerpAction::new(LerpConfig {
                target: camera,
                lerp_type: LerpType::Position(CAMERA_OFFSET * 0.5),
                duration: 1.0,
            })
            .into_boxed(),
            LerpAction::new(LerpConfig {
                target: state.agent,
                lerp_type: LerpType::Rotation(Quat::from_look(Vec3::Z, Vec3::Y)),
                duration: 1.0,
            })
            .into_boxed(),
            WaitAction::new(1.0).into_boxed(),
            LerpAction::new(LerpConfig {
                target: camera,
                lerp_type: LerpType::Position(CAMERA_OFFSET),
                duration: 1.0,
            })
            .into_boxed(),
            WaitAction::new(0.5).into_boxed(),
            LerpAction::new(LerpConfig {
                target: state.agent,
                lerp_type: LerpType::Rotation(Quat::from_look(-Vec3::Z, Vec3::Y)),
                duration: 1.0,
            })
            .into_boxed(),
        ];

        commands
            .actions(state.agent)
            .config(AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: Repeat::Amount(0),
            })
            .add_many(ExecutionMode::Sequential, actions.into_iter())
            .next();
    }

    fn on_stop(&mut self, _state: &mut WorldState, _reason: StopReason) {}
}

struct FancyAction;

impl Action for FancyAction {
    fn on_start(&mut self, state: &mut WorldState, commands: &mut ActionCommands) {
        // This action runs a system that adds another wait action.
        // When modifying actions using world inside the Action trait,
        // it is important that the modifications happens after the on_start method.
        // Use the custom method for deferred world mutation.

        let agent = state.agent;
        commands.custom(move |world| {
            world.run_system(my_system);
            world.actions(agent).next();
        });
    }

    fn on_stop(&mut self, _state: &mut WorldState, _reason: StopReason) {}
}

fn my_system(agent_q: Query<Entity, With<ActionMarker>>, mut commands: Commands) {
    let agent = agent_q.single();
    commands
        .actions(agent)
        .config(AddConfig {
            order: AddOrder::Front,
            ..Default::default()
        })
        .add(WaitAction::new(1.0));
}
