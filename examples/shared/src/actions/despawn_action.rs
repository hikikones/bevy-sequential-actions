use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl DespawnAction {
    pub fn new() -> Self {
        Self
    }
}

impl Action for DespawnAction {
    fn start(
        &mut self,
        state: StartState,
        entity: Entity,
        world: &mut World,
        commands: &mut ActionCommands,
    ) {
        world.despawn(entity);
    }

    fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {}
}
