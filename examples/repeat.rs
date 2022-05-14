use bevy::prelude::*;

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

    // Add three wait actions with custom config
    commands
        .action(entity)
        .config(AddConfig {
            order: AddOrder::Back, // Add each action to the back of the queue
            start: true,           // Start action if nothing is currently running
            repeat: true,          // Repeat each action when it has finished
        })
        .add(WaitAction(1.0))
        .add(WaitAction(2.0))
        .add(WaitAction(3.0));

    // These three wait actions will now basically loop forever in the added order.
}

struct WaitAction(f32);

impl Action for WaitAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
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
