use bevy::ecs::{entity::Entity, world::World};
use bevy_sequential_actions::*;

/// An action that runs a sequence of actions.
/// Useful when you want a sequence to act as a single action.
/// For example, canceling this action will cancel the entire sequence.
pub struct ActionSequence<const N: usize> {
    actions: [BoxedAction; N],
    index: usize,
}

impl<const N: usize> ActionSequence<N> {
    pub const fn new(actions: [BoxedAction; N]) -> Self {
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

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.actions[self.index].on_stop(agent, world, reason);

        if reason == StopReason::Canceled {
            self.index = N;
        }
    }

    fn on_drop(mut self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        self.index += 1;

        if self.index >= N || reason != DropReason::Done {
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        }

        let Some(agent) = agent else { return };

        world
            .get_mut::<ActionQueue>(agent)
            .unwrap()
            .push_front(self);
    }
}

/// An action like [`ActionSequence`] but also repeats like [`RepeatAction`](crate::RepeatAction).
pub struct RepeatActionSequence<const N: usize> {
    actions: [BoxedAction; N],
    index: usize,
    repeat: u32,
}

impl<const N: usize> RepeatActionSequence<N> {
    pub const fn new(actions: [BoxedAction; N], repeat: u32) -> Self {
        Self {
            actions,
            index: 0,
            repeat,
        }
    }
}

impl<const N: usize> Action for RepeatActionSequence<N> {
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

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.actions[self.index].on_stop(agent, world, reason);

        if reason == StopReason::Canceled {
            self.index = N;
        }
    }

    fn on_drop(mut self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        self.index += 1;

        if reason != DropReason::Done {
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        }

        if self.index >= N {
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));

            if self.repeat == 0 {
                return;
            }

            let Some(agent) = agent else { return };

            self.repeat -= 1;
            self.index = 0;
            world
                .actions(agent)
                .config(AddConfig::new(false, AddOrder::Front))
                .add(self as BoxedAction);
            return;
        }

        let Some(agent) = agent else { return };

        world
            .get_mut::<ActionQueue>(agent)
            .unwrap()
            .push_front(self);
    }
}
