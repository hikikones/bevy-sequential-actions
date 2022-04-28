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
        let mut state = world.get_resource_mut::<State<T>>().unwrap();
        state.set(self.0).unwrap();
        commands.next_action(actor);
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}
