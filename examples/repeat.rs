use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugin(ScheduleRunnerPlugin)
        .add_plugin(SequentialActionsPlugin::default())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::default()).id();
    commands
        .actions(agent)
        .add(RepeatAction {
            print_message: "hello",
            repeat_count: 3,
        })
        .add(RepeatAction {
            print_message: "world",
            repeat_count: 1,
        });
}

struct RepeatAction {
    print_message: &'static str,
    repeat_count: u32,
}

impl Action for RepeatAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {
        println!("{}", self.print_message);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}

    fn on_remove(mut self: Box<Self>, agent: Entity, world: &mut World) {
        if self.repeat_count == 0 {
            if world.get::<ActionQueue>(agent).unwrap().is_empty() {
                world.send_event(AppExit);
            }
            return;
        }

        self.repeat_count -= 1;
        world.actions(agent).start(false).add(self as BoxedAction);
    }
}
