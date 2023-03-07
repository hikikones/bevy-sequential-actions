use bevy::{ecs::schedule::States, prelude::*};
use bevy_sequential_actions::*;

pub struct SetStateAction<S: States>(S);

impl<S: States> SetStateAction<S> {
    pub fn new(state: S) -> Self {
        Self(state)
    }
}

impl<S: States> Action for SetStateAction<S> {
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        world.resource_mut::<NextState<S>>().set(self.0.clone());

        commands.actions(agent).next();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
