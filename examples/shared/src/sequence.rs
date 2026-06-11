use bevy::ecs::{entity::Entity, world::World};
use bevy_sequential_actions::*;

/// An action that runs a sequence of actions.
/// Useful when you want a sequence to act as a single action.
/// For example, canceling this action will drop the entire sequence.
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
            // We have finished the sequence or action has been skipped or cleared.
            // Time for removal.
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        }

        let Some(agent) = agent else {
            // We are not done with the sequence, but agent is non-existent for some reason.
            // Not much we can do about that, so time for removal.
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        };

        // We are not done with the entire sequence yet, so we add it back again to the front.
        // We also push it directly to the action queue to avoid calling `on_add` again.
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
            // Action has been skipped or cleared, time for removal.
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        }

        if self.index >= N {
            // We have finished the sequence, time for removal.
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));

            if self.repeat == 0 {
                // No more repeats. We are completely done, so simply return.
                return;
            }

            let Some(agent) = agent else {
                // We are not yet done repeating, but agent is non-existent for some reason.
                // Not much we can do about that, so simply return.
                return;
            };

            // Decrement repeat and reset index
            self.repeat -= 1;
            self.index = 0;

            // We are not done with all repeats yet, so we add it back again to the front.
            // Since we have called the `on_remove` on each action,
            // we add it back the usual way so `on_add` is also called again.
            world
                .actions(agent)
                .start(false)
                .order(AddOrder::Front)
                .add(self as BoxedAction);
            return;
        }

        let Some(agent) = agent else {
            // We are not done with the sequence, but agent is non-existent for some reason.
            // Not much we can do about that, so time for removal.
            self.actions
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
            return;
        };

        // We are not done with the entire sequence yet, so we add it back again to the front.
        // We also push it directly to the action queue to avoid calling `on_add` again.
        world
            .get_mut::<ActionQueue>(agent)
            .unwrap()
            .push_front(self);
    }
}
