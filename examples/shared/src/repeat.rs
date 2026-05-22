use bevy::ecs::{entity::Entity, world::World};
use bevy_sequential_actions::*;

pub struct RepeatAction<A: Action> {
    action: A,
    repeat: u32,
}

impl<A: Action> RepeatAction<A> {
    pub const fn new(action: A, repeat: u32) -> Self {
        Self { action, repeat }
    }
}

impl<A: Action> Action for RepeatAction<A> {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        self.action.is_finished(agent, world)
    }

    fn on_add(&mut self, agent: Entity, world: &mut World) {
        self.action.on_add(agent, world);
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        self.action.on_start(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        self.action.on_stop(agent, world, reason);
    }

    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {
        self.action.on_remove(agent, world);
    }

    fn on_drop(mut self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {
        if self.repeat == 0 || reason != DropReason::Done {
            return;
        }

        let Some(agent) = agent else { return };

        self.repeat -= 1;
        world.actions(agent).start(false).add(self as BoxedAction);
    }
}
