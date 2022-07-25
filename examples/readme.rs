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
    commands
        .actions(entity)
        .add(PrintAction("Hello there".into()));

    // Add multiple actions with custom config
    commands
        .actions(entity)
        .config(AddConfig {
            // Add each action to the back of the queue
            order: AddOrder::Back,
            // Start action if nothing is currently running
            start: true,
            // Repeat the action
            repeat: false,
        })
        .add(WaitAction(1.0))
        .add(PrintAction("That's".into()))
        .add(PrintAction("all,".into()))
        .add(PrintAction("folks!".into()))
        .add(QuitAction);
}

struct PrintAction(String);

impl Action for PrintAction {
    fn start(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        println!("{}", self.0);
        // Action is finished, issue next.
        commands.actions(entity).next();
    }

    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}

struct WaitAction(f32);

impl Action for WaitAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("Wait({})", self.0);
        world.entity_mut(entity).insert(Wait(self.0));
    }

    fn stop(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<Wait>();
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
    for (entity, mut wait) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            // Action is finished, issue next.
            commands.actions(entity).next();
        }
    }
}

struct QuitAction;

impl Action for QuitAction {
    fn start(&mut self, _entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}
