use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl Action for DespawnAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(entity).despawn_recursive();
    }

    fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
}
