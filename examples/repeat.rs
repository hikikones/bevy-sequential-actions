use bevy::prelude::*;
use bevy_sequential_actions::*;

use shared::actions::wait_action::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(WaitActionPlugin)
        .add_startup_system(setup)
        // .add_system(wait)
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
        .add(WaitAction::new(1.0))
        .add(WaitAction::new(2.0))
        .add(WaitAction::new(3.0));

    // These three wait actions will now basically loop forever in the added order.
}
