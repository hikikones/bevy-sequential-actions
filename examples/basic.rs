use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::actions::{QuitAction, WaitAction, WaitActionPlugin};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(WaitActionPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let entity = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single action with default config
    commands.action(entity).add(WaitAction::new(1.0));

    // Add multiple actions with custom config
    commands
        .action(entity)
        .config(AddConfig {
            order: AddOrder::Back, // Add each action to the back of the queue
            start: false,          // Start action if nothing is currently running
            repeat: false,         // Repeat the action
        })
        .add(WaitAction::new(4.0))
        .add(WaitAction::new(5.0));

    // Push multiple actions to the front and reverse order before submitting
    commands
        .action(entity)
        .config(AddConfig {
            order: AddOrder::Front, // This time, add each action to the front
            start: false,
            repeat: false,
        })
        .push(WaitAction::new(2.0))
        .push(WaitAction::new(3.0))
        .reverse() // Reverse the order to get increasing wait times
        .submit(); // When pushing, actions are not queued until submit is called.

    // Add an action that itself adds multiple actions
    commands.action(entity).add(MultipleWaitActions);

    // Finally, quit the app
    commands.action(entity).add(QuitAction::new());

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

struct MultipleWaitActions;

impl Action for MultipleWaitActions {
    fn start(&mut self, entity: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // This action simply creates new actions to the front of the queue.
        commands
            .action(entity)
            .config(AddConfig {
                order: AddOrder::Front,
                start: false,
                repeat: false,
            })
            .push(WaitAction::new(4.0))
            .push(WaitAction::new(3.0))
            .push(WaitAction::new(2.0))
            .push(WaitAction::new(1.0))
            .reverse()
            .submit()
            .next(); // Since this is all that it does, we call next action as it is finished.
    }

    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}
