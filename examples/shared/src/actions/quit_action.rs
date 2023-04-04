use bevy::{
    app::AppExit,
    ecs::{entity::Entity, event::Events, world::World},
};
use bevy_sequential_actions::*;

pub struct QuitAction;

impl Action for QuitAction {
    fn on_start(&mut self, _agent: Entity, world: &mut World) {
        world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
