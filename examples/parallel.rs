use bevy::prelude::*;
use bevy_sequential_actions::*;
use shared::{CountdownAction, ParallelActions, PrintAction, SharedActionsPlugin, WaitAction};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, SequentialActionsPlugin, SharedActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands.actions(agent).add((
        ParallelActions::new(actions![
            WaitAction::new(0.001),
            CountdownAction::new(10),
            PrintAction::new("hello there"),
        ]),
        |_agent, world: &mut World| {
            world.write_message(AppExit::Success);
            false
        },
    ));
}
