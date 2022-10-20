use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl Action for DespawnAction {
    fn on_start(&mut self, id: ActionEntities, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(id.agent()).despawn_recursive();
    }

    fn on_stop(&mut self, _id: ActionEntities, _world: &mut World, _reason: StopReason) {}
}
