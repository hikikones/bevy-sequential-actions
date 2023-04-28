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
        .add_many(actions![
            RepeatAction {
                repeat_action: PrintAction("hello"),
                repeat_count: 3,
            },
            RepeatAction {
                repeat_action: PrintAction("world"),
                repeat_count: 1,
            },
        ])
        .add(|_agent, world: &mut World| {
            world.send_event(AppExit);
        });
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

struct RepeatAction<A: Action> {
    repeat_action: A,
    repeat_count: u32,
}

impl<A: Action> Action for RepeatAction<A> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.repeat_action.is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.repeat_action.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        self.repeat_action.on_start(agent, world);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.repeat_action.on_stop(agent, world, reason);
    }

    fn on_drop(mut self: Box<Self>, agent: Entity, world: &mut World) {
        if self.repeat_count == 0 {
            self.repeat_action.on_remove(agent, world);
            return;
        }

        self.repeat_count -= 1;
        world.get_mut::<ActionQueue>(agent).unwrap().push_back(self);
    }
}
