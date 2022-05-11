use bevy::prelude::*;

use bevy_sequential_actions::*;

use crate::bevy_extensions::*;

pub(super) struct SetStateActionPlugin;

impl Plugin for SetStateActionPlugin {
    fn build(&self, _app: &mut App) {}
}

pub struct SetStateAction<T: BevyState>(T);

impl<T: BevyState> SetStateAction<T> {
    pub fn new(state: T) -> Self {
        Self(state)
    }
}

impl<T: BevyState> Action for SetStateAction<T> {
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
        world.set_state(self.0);
        commands.action(actor).next();
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}
    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}
