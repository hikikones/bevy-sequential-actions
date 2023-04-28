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
    commands.actions(agent).add_many(actions![
        PrintRepeatAction {
            print_action: PrintAction("hello"),
            repeat_count: 3,
        },
        PrintRepeatAction {
            print_action: PrintAction("world"),
            repeat_count: 1,
        },
    ]);
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) {
        println!("{}", self.0);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

struct PrintRepeatAction {
    print_action: PrintAction,
    repeat_count: u32,
}

impl Action for PrintRepeatAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.print_action.is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.print_action.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        self.print_action.on_start(agent, world);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.print_action.on_stop(agent, world, reason);
    }

    fn on_drop(mut self: Box<Self>, agent: Entity, world: &mut World) {
        if self.repeat_count == 0 {
            self.print_action.on_remove(agent, world);

            if world.get::<ActionQueue>(agent).unwrap().is_empty() {
                world.send_event(AppExit);
            }

            return;
        }

        self.repeat_count -= 1;
        world.get_mut::<ActionQueue>(agent).unwrap().push_back(self);
    }
}
