use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl Action for DespawnAction {
    fn on_start(&mut self, agent: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // Despawn in a deferred way, and do not advance the actions queue.
        commands.custom(move |w: &mut World| {
            w.entity_mut(agent).despawn_recursive();
        });
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

pub struct RemoveActionsBundleAction;

impl Action for RemoveActionsBundleAction {
    fn on_start(&mut self, agent: Entity, _world: &mut World, commands: &mut ActionCommands) {
        // Remove bundle in a deferred way, and do not advance the actions queue.
        commands.custom(move |w: &mut World| {
            w.entity_mut(agent).remove_bundle::<ActionsBundle>();
        });
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
