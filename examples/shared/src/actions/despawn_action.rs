use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl DespawnAction {
    pub fn new() -> Self {
        Self
    }
}

impl Action for DespawnAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.despawn(entity);
    }

    fn stop(&mut self, _entity: Entity, _world: &mut World) {}
}
