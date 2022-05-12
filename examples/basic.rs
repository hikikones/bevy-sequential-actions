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
    let entity = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single action with default config
    commands.action(entity).add(WaitAction(1.0));

    // Add multiple actions with custom config
    commands
        .action(entity)
        .config(AddConfig {
            order: AddOrder::Back, // Add each action to the back of the queue
            start: false,          // Start action if nothing is currently running
            repeat: false,         // Repeat the action
        })
        .add(WaitAction(4.0))
        .add(WaitAction(5.0));

    // Push multiple actions to the front and reverse order before submitting
    commands
        .action(entity)
        .config(AddConfig {
            order: AddOrder::Front, // This time, add each action to the front
            start: false,
            repeat: false,
        })
        .push(WaitAction(2.0))
        .push(WaitAction(3.0))
        .reverse() // Reverse the order to get increasing wait times
        .submit(); // When pushing, actions are not queued until submit is called.

    // Add an action that itself adds multiple actions
    commands.action(entity).add(MultipleWaitActions);

    // Finally, quit the app
    commands.action(entity).add(QuitAction);

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
    fn add(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("Wait({})", self.0);
        world.entity_mut(entity).insert(Wait(self.0));
    }

    fn remove(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<Wait>();
    }

    fn stop(&mut self, entity: Entity, world: &mut World) {
        self.remove(entity, world);
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
    for (entity, mut wait) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            // To signal that an action has finished, the next action method must be called.
            commands.action(entity).next();
        }
    }
}

struct MultipleWaitActions;

impl Action for MultipleWaitActions {
    fn add(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // This action simply creates new actions to the front of the queue.
        commands
            .action(entity)
            .config(AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: false,
            })
            .push(WaitAction(4.0))
            .push(WaitAction(3.0))
            .push(WaitAction(2.0))
            .push(WaitAction(1.0))
            .reverse()
            .submit()
            .next(); // Since this is all that it does, we call next action as it is finished.
    }

    fn remove(&mut self, _entity: Entity, _world: &mut World) {}
    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}

struct QuitAction;

impl Action for QuitAction {
    fn add(&mut self, _entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("Quit");
        let mut app_exit_ev = world.resource_mut::<Events<AppExit>>();
        app_exit_ev.send(AppExit);
    }

    fn remove(&mut self, _entity: Entity, _world: &mut World) {}
    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}
