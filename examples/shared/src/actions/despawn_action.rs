use bevy::prelude::*;
use bevy_sequential_actions::*;

pub struct DespawnAction;

impl Action for DespawnAction {
    fn on_start(&mut self, agent: Entity, world: &mut World) {
        // Despawn in a deferred way, and do not advance the action queue.
        world.deferred_actions(agent).custom(move |w: &mut World| {
            w.entity_mut(agent).despawn_recursive();
        });
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

pub struct RemoveActionsBundleAction;

impl Action for RemoveActionsBundleAction {
    fn on_start(&mut self, agent: Entity, world: &mut World) {
        // Remove bundle in a deferred way, and do not advance the action queue.
        world.deferred_actions(agent).custom(move |w: &mut World| {
            w.entity_mut(agent).remove::<ActionsBundle>();
        });
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
