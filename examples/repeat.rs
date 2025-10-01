use bevy_app::{AppExit, ScheduleRunnerPlugin, prelude::*};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((ScheduleRunnerPlugin::default(), SequentialActionsPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let agent = commands.spawn(SequentialActions).id();
    commands.actions(agent).add((
        RepeatAction {
            action: PrintAction("hello"),
            repeat: 3,
        },
        RepeatAction {
            action: PrintAction("world"),
            repeat: 1,
        },
        RepeatAction {
            action: |agent, world: &mut World| {
                // Exit app when action queue is empty
                if world.get::<ActionQueue>(agent).unwrap().is_empty() {
                    world.write_message(AppExit::Success);
                }

                // Do not advance action queue immediately,
                // otherwise we get stuck in an infinite loop
                // as we keep readding this action
                false
            },
            repeat: u32::MAX,
        },
    ));
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

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.action.on_stop(agent, world, reason);
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        self.action.on_remove(agent, world);
    }

    fn on_drop(mut self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        if self.repeat == 0 || reason != DropReason::Done {
            return;
        }

        let Some(agent) = agent else { return };

        self.repeat -= 1;
        world.actions(agent).start(false).add(self as BoxedAction);
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

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}
