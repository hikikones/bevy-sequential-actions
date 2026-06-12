use bevy::prelude::*;
use bevy_sequential_actions::*;
use shared::{CountdownAction, PrintAction, SharedActionsPlugin};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, SequentialActionsPlugin, SharedActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn entity with the marker component
    let agent = commands.spawn(SequentialActions).id();
    commands
        .actions(agent)
        // Add a single action
        .add(DemoAction)
        // Add more actions with a tuple
        .add((
            PrintAction::new("hello"),
            PrintAction::new("there"),
            CountdownAction::new(5),
        ))
        // Add a collection of actions
        .add(actions![
            PrintAction::new("it is possible to commit no mistakes and still lose"),
            PrintAction::new("that is not a weakness"),
            PrintAction::new("that is life"),
            CountdownAction::new(10),
        ])
        // Add an anonymous action with a closure
        .add(|_agent, world: &mut World| -> bool {
            // on_start
            world.write_message(AppExit::Success);
            true
        });
}

struct DemoAction;

impl Action for DemoAction {
    // Required method
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        println!("is_finished: called every frame in the Last schedule");
        true
    }

    // Required method
    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("on_start: called when an action is started");

        // Returning true here marks the action as already finished,
        // and will immediately advance the action queue.
        false
    }

    // Required method
    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {
        println!("on_stop: called when an action is stopped");
    }

    // Optional method (empty by default)
    fn on_add(&mut self, _agent: Entity, _world: &mut World) {
        println!("on_add: called when an action is added to the queue");
    }

    // Optional method (empty by default)
    fn on_remove(&mut self, _agent: Option<Entity>, _world: &mut World) {
        println!("on_remove: called when an action is removed from the queue");
    }

    // Optional method (empty by default)
    fn on_drop(self: Box<Self>, _agent: Option<Entity>, _world: &mut World, _reason: DropReason) {
        println!("on_drop: the last method to be called with full ownership");
    }
}
