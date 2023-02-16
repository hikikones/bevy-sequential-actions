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
    let agent = commands.spawn_agent(AgentConfig::default());

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
            // Repeat the action
            repeat: Repeat::None,
        })
        .add(MoveAction::new(MoveConfig {
            target: -Vec3::X * 2.0,
            speed: 4.0,
            rotate: true,
        }))
        .add(WaitAction::new(1.0))
        .add(MoveAction::new(MoveConfig {
            target: Vec3::ZERO,
            speed: 4.0,
            rotate: true,
        }))
        .add(WaitAction::new(1.0));

    // Add a collection of actions
    commands
        .actions(agent)
        .config(AddConfig {
            // This time, add each action to the front of the queue
            order: AddOrder::Front,
            ..Default::default()
        })
        .add_sequence(actions![
            WaitAction::new(10.0),
            WaitAction::new(100.0),
            WaitAction::new(1000.0),
        ])
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
        .add(
            |agent: Entity, _world: &mut World, commands: &mut ActionCommands| {
                // on_start
                commands.actions(agent).next();
            },
        )
        // Tuple closure for both the on_start and on_stop methods
        .add((
            |agent: Entity, _world: &mut World, commands: &mut ActionCommands| {
                // on_start
                commands.actions(agent).next();
            },
            |_agent: Entity, _world: &mut World, _reason: StopReason| {
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
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        // This action adds a bunch of other actions to the front.
        // Since this is all that it does, we call next() at the end.

        let camera = world
            .query_filtered::<Entity, With<CameraMain>>()
            .single(world);

        commands
            .actions(agent)
            .config(AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: Repeat::None,
            })
            .add_sequence(actions![
                LerpAction::new(LerpConfig {
                    target: camera,
                    lerp_type: LerpType::Position(CAMERA_OFFSET * 0.5),
                    duration: 1.0,
                }),
                LerpAction::new(LerpConfig {
                    target: agent,
                    lerp_type: LerpType::Rotation(Quat::from_look(Vec3::Z, Vec3::Y)),
                    duration: 1.0,
                }),
                WaitAction::new(1.0).into_boxed(),
                LerpAction::new(LerpConfig {
                    target: camera,
                    lerp_type: LerpType::Position(CAMERA_OFFSET),
                    duration: 1.0,
                }),
                WaitAction::new(0.5).into_boxed(),
                LerpAction::new(LerpConfig {
                    target: agent,
                    lerp_type: LerpType::Rotation(Quat::from_look(-Vec3::Z, Vec3::Y)),
                    duration: 1.0,
                }),
            ])
            .next();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct FancyAction;

impl Action for FancyAction {
    fn on_start(&mut self, agent: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // This action runs a system that adds another wait action.
        // When modifying actions using world inside the Action trait,
        // it is important that the modifications happens after the on_start method.

        // Add a custom command for deferred world mutation.
        commands.add(move |world| {
            world.run_system(my_system);
            world.actions(agent).next();
        });
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

fn my_system(agent_q: Query<Entity, With<Agent>>, mut commands: Commands) {
    let agent = agent_q.single();
    commands
        .actions(agent)
        .config(AddConfig {
            order: AddOrder::Front,
            ..Default::default()
        })
        .add(WaitAction::new(1.0));
}
