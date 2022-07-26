use bevy::{app::AppExit, ecs::event::Events, prelude::*};
use bevy_sequential_actions::*;

pub struct QuitAction;

impl QuitAction {
    pub fn new() -> Self {
        Self
    }
}

impl Action for QuitAction {
    fn start(&mut self, _entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn finish(&mut self, _entity: Entity, _world: &mut World) {}
}
