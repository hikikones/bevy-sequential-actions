use bevy::{app::AppExit, ecs::event::Events, prelude::*};
use bevy_sequential_actions::*;

pub struct QuitAction;

impl QuitAction {
    pub fn new() -> Self {
        Self
    }
}

impl Action for QuitAction {
    fn start(
        &mut self,
        state: StartState,
        entity: Entity,
        world: &mut World,
        commands: &mut ActionCommands,
    ) {
        world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {}
}
