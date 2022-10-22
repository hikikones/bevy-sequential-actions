use bevy::{ecs::schedule::StateData, prelude::*};
use bevy_sequential_actions::*;

pub struct SetStateAction<T: StateData>(T);

impl<T: StateData> SetStateAction<T> {
    pub fn new(state: T) -> Self {
        Self(state)
    }
}

impl<T: StateData> Action for SetStateAction<T> {
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        world
            .resource_mut::<State<T>>()
            .set(self.0.clone())
            .unwrap();

        commands.actions(agent).next();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
