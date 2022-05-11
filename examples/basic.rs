use bevy::{app::AppExit, ecs::event::Events, prelude::*};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_startup_system(setup)
        .add_system(wait)
        .run();
}

fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let id = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single action with default config
    commands.action(id).add(WaitAction(1.0));

    // Add multiple actions with custom config
    commands
        .action(id)
        .config(AddConfig {
            order: AddOrder::Back, // Add each action to the back of the queue
            start: false,          // Start action if nothing is currently running
            repeat: false,         // Repeat the action
        })
        .push(WaitAction(4.0))
        .push(WaitAction(5.0))
        .submit();

    // Add multiple actions again but to the front of the queue
    commands
        .action(id)
        .config(AddConfig {
            order: AddOrder::Front, // This time, add each action to the front of the queue
            start: false,
            repeat: false,
        })
        .push(WaitAction(2.0))
        .push(WaitAction(3.0))
        .reverse() // Since we are adding to the front, reverse the order to get increasing wait times
        .submit();

    // Add an action that itself adds multiple actions
    commands.action(id).add(MultipleWaitActions);

    // Finally, quit the app
    commands.action(id).add(QuitAction);

    // A list of actions have now been added to the queue, and should execute in the following order:
    // Wait(1.0)
    // Wait(2.0)
    // Wait(3.0)
    // Wait(4.0)
    // Wait(5.0)
    // Wait(4.0)
    // Wait(3.0)
    // Wait(2.0)
    // Wait(1.0)
    // Quit
}

struct WaitAction(f32);

impl Action for WaitAction {
    fn add(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("Wait({})", self.0);
        world.entity_mut(actor).insert(Wait(self.0));
    }

    fn remove(&mut self, actor: Entity, world: &mut World) {
        world.entity_mut(actor).remove::<Wait>();
    }

    fn stop(&mut self, actor: Entity, world: &mut World) {
        self.remove(actor, world);
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
    for (actor, mut wait) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            // To signal that an action has finished, the next action method must be called.
            commands.action(actor).next();
        }
    }
}

struct MultipleWaitActions;

impl Action for MultipleWaitActions {
    fn add(&mut self, actor: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // This action simply creates new actions to the front of the queue.
        commands
            .action_builder(
                actor,
                AddConfig {
                    order: AddOrder::Front,
                    start: false,
                    repeat: false,
                },
            )
            .push(WaitAction(4.0))
            .push(WaitAction(3.0))
            .push(WaitAction(2.0))
            .push(WaitAction(1.0))
            .reverse()
            .submit();

        // Since this is all that it does, we call next action as it is finished.
        commands.next_action(actor);
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}

struct QuitAction;

impl Action for QuitAction {
    fn add(&mut self, _actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("Quit");
        let mut app_exit_ev = world.resource_mut::<Events<AppExit>>();
        app_exit_ev.send(AppExit);
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}
