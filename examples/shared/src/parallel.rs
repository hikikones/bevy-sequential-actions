use bevy::ecs::{entity::Entity, world::World};
use bevy_sequential_actions::*;

/// An action that runs multiple actions in parallel.
/// Will only advance when all actions are finished within
/// the same frame.
pub struct ParallelActions<const N: usize> {
    actions: [BoxedAction; N],
}

impl<const N: usize> ParallelActions<N> {
    pub const fn new(actions: [BoxedAction; N]) -> Self {
        Self { actions }
    }
}

impl<const N: usize> Action for ParallelActions<N> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.actions
            .iter()
            .all(|action| action.is_finished(agent, world))
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_add(agent, world));
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        std::array::from_fn::<bool, N, _>(|i| self.actions[i].on_start(agent, world))
            .into_iter()
            .all(|b| b)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_stop(agent, world, reason));
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        self.actions
            .iter_mut()
            .for_each(|action| action.on_remove(agent, world));
    }
}
