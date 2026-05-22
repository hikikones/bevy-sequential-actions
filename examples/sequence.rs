use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_sequential_actions::*;
use shared::{ActionSequence, PrintAction};

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands.actions(agent).add(ActionSequence::new(actions![
        PrintAction::new("see"),
        PrintAction::new("you"),
        PrintAction::new("in"),
        PrintAction::new("space"),
        PrintAction::new("cowboy"),
        |_agent, world: &mut World| -> bool {
            world.write_message(AppExit::Success);
            true
        }
    ]));
}
