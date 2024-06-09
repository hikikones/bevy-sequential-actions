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
    commands.actions(agent).add(ActionSequence::new(actions![
        PrintAction("see"),
        PrintAction("you"),
        PrintAction("in"),
        PrintAction("space"),
        PrintAction("cowboy"),
        |_agent, world: &mut World| -> bool {
            world.send_event(AppExit::Success);
            false
        }
    ]));
}

struct ActionSequence<const N: usize> {
    actions: [BoxedAction; N],
    index: usize,
}

impl<const N: usize> ActionSequence<N> {
    fn new(actions: [BoxedAction; N]) -> Self {
        Self { actions, index: 0 }
    }
}

impl<const N: usize> Action for ActionSequence<N> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.actions[self.index].is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_add(agent, world));
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        self.actions[self.index].on_start(agent, world)
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        self.actions[self.index].on_stop(agent, world, reason);

        if reason == StopReason::Canceled {
            self.index = self.actions.len();
        }
    }

    fn on_drop(mut self: Box<Self>, agent: Entity, world: &mut World, reason: DropReason) {
        self.index += 1;

        if self.index >= self.actions.len() || reason != DropReason::Done {
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
        } else {
            world
                .get_mut::<ActionQueue>(agent)
                .unwrap()
                .push_front(self);
        }
    }
}

struct PrintAction(&'static str);

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("{}", self.0);
        false
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
