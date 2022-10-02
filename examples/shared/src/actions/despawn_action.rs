use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl Action for DespawnAction {
    fn on_start(&mut self, state: &mut WorldState, _commands: &mut ActionCommands) {
        state.world.entity_mut(state.agent).despawn_recursive();
    }

    fn on_stop(&mut self, _state: &mut WorldState, _reason: StopReason) {}
}
