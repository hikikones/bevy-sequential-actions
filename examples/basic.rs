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
    commands.add_action(id, WaitAction(1.0), AddConfig::default());

    // Add multiple actions with custom config
    commands
        .action_builder(
            id,
            AddConfig {
                order: AddOrder::Back, // Add action to the back of the queue
                start: false,          // Start the action if nothing is currently running
                repeat: false, // Repeat the action by adding it back the queue after finishing
            },
        )
        .add(WaitAction(4.0))
        .add(WaitAction(5.0))
        .submit();

    // Add multiple actions again but to the front of the queue
    commands
        .action_builder(
            id,
            AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: false,
            },
        )
        .add(WaitAction(2.0))
        .add(WaitAction(3.0))
        .reverse() // Reverse add order to get increasing wait times
        .submit();

    // Finally, quit the app
    commands.add_action(id, QuitAction, AddConfig::default());

    // A queue of actions have been added and should execute in the following order:
    // Wait(1.0)
    // Wait(2.0)
    // Wait(3.0)
    // Wait(4.0)
    // Wait(5.0)
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
            commands.next_action(actor);
        }
    }
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
