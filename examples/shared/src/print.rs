use bevy::ecs::{entity::Entity, world::World};
use bevy_sequential_actions::*;

pub struct PrintAction(&'static str);

impl PrintAction {
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }
}

impl Action for PrintAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        println!("{}", self.0);
        true
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}
