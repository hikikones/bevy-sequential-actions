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
        .add(|_agent, world: &mut World| -> Finished {
            world.send_event(AppExit);
            Finished(false)
        });
}

struct RepeatAction<A: Action> {
    action: A,
    repeat: u32,
}

impl<A: Action> Action for RepeatAction<A> {
    fn is_finished(&self, agent: Entity, world: &World) -> Finished {
        self.action.is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.action.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> Finished {
        self.action.on_start(agent, world)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.action.on_stop(agent, world, reason);
    }

    fn on_drop(mut self: Box<Self>, agent: Entity, world: &mut World) {
        if self.repeat == 0 {
            self.action.on_remove(agent, world);
            return;
        }

        self.repeat -= 1;
        world.get_mut::<ActionQueue>(agent).unwrap().push_back(self);
    }
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> Finished {
        Finished(true)
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> Finished {
        println!("{}", self.0);
        Finished(true)
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
