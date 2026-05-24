use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_sequential_actions::*;
use shared::{PrintAction, RepeatAction};

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands.actions(agent).add((
        RepeatAction::new(PrintAction::new("zero"), 0),
        RepeatAction::new(PrintAction::new("one"), 1),
        RepeatAction::new(PrintAction::new("two"), 2),
        RepeatAction::new(
            |agent, world: &mut World| -> bool {
                // Exit app when action queue is empty
                if world.get::<ActionQueue>(agent).unwrap().is_empty() {
                    world.write_message(AppExit::Success);
                }

                // Do not advance action queue immediately,
                // otherwise we get stuck in an infinite loop
                // as we keep readding this action
                false
            },
            u32::MAX,
        ),
    ));
}
