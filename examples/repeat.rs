use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(ActionsBundle::new()).id();
    commands
        .actions(agent)
        .add_many(actions![
            RepeatAction {
                action: PrintAction("hello"),
                repeat: 3,
            },
            RepeatAction {
                action: PrintAction("world"),
                repeat: 1,
            },
        ])
        .add(|_agent, world: &mut World| -> bool {
            world.send_event(AppExit);
            false
        });
}

struct RepeatAction<A: Action> {
    action: A,
    repeat: u32,
}

impl<A: Action> Action for RepeatAction<A> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.action.is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.action.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        self.action.on_start(agent, world)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.action.on_stop(agent, world, reason);
    }

    fn on_drop(mut self: Box<Self>, agent: Entity, world: &mut World, reason: DropReason) {
        if self.repeat == 0 || reason != DropReason::Done {
            self.action.on_remove(agent, world);
            return;
        }

        self.repeat -= 1;
        world.get_mut::<ActionQueue>(agent).unwrap().push_back(self);
    }
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("{}", self.0);
        true
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
