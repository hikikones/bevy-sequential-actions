use bevy::{app::AppExit, ecs::event::Events, prelude::*};

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let entity = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single guess action with callback
    commands
        .action(entity)
        .add(GuessAction::new(Some(callback)));
}

type CallbackFn = fn(entity: Entity, success: bool, commands: &mut ActionCommands);

fn callback(entity: Entity, success: bool, commands: &mut ActionCommands) {
    // Quit app if success
    if success {
        println!("Success!");
        commands.action(entity).add(QuitAction);
        return;
    }

    // Guess again
    commands
        .action(entity)
        .add(GuessAction::new(Some(callback)));
}

struct GuessAction {
    callback: Option<CallbackFn>,
}

impl GuessAction {
    fn new(callback: Option<CallbackFn>) -> Self {
        Self { callback }
    }
}

impl Action for GuessAction {
    fn start(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        let random = fastrand::u8(0..9);
        let guess = fastrand::u8(0..9);

        println!("Random: {random} \t Guess: {guess}");

        if let Some(callback) = self.callback {
            let success = guess == random;
            callback(entity, success, commands);
        }

        commands.action(entity).next();
    }

    fn remove(&mut self, _entity: Entity, _world: &mut World) {}
    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}

struct QuitAction;

impl Action for QuitAction {
    fn start(&mut self, _entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        println!("Quit");
        let mut app_exit_ev = world.resource_mut::<Events<AppExit>>();
        app_exit_ev.send(AppExit);
    }

    fn remove(&mut self, _entity: Entity, _world: &mut World) {}
    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}
