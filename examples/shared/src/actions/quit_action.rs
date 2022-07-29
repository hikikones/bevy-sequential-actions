use bevy::{app::AppExit, ecs::event::Events, prelude::*};
use bevy_sequential_actions::*;

pub struct QuitAction;

impl QuitAction {
    pub fn new() -> Self {
        Self
    }
}

impl Action for QuitAction {
    fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn on_finish(&mut self, entity: Entity, world: &mut World) {}
    fn on_cancel(&mut self, entity: Entity, world: &mut World) {}
    fn on_stop(&mut self, entity: Entity, world: &mut World) {}
}
