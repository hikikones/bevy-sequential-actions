use std::collections::VecDeque;

use bevy_ecs::prelude::*;

mod action_commands;
mod commands;
mod traits;

pub mod world;

pub use action_commands::*;
pub use commands::*;
pub use traits::*;

pub trait Action: Send + Sync {
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands);
    fn remove(&mut self, actor: Entity, world: &mut World);
    fn stop(&mut self, actor: Entity, world: &mut World);
}

#[derive(Default, Bundle)]
pub struct ActionsBundle {
    queue: ActionQueue,
    current: CurrentAction,
}

#[derive(Clone, Copy)]
pub enum AddOrder {
    Back,
    Front,
}

#[derive(Clone, Copy)]
pub struct AddConfig {
    pub order: AddOrder,
    pub start: bool,
    pub repeat: bool,
}

impl Default for AddConfig {
    fn default() -> Self {
        Self {
            order: AddOrder::Back,
            start: true,
            repeat: false,
        }
    }
}

#[derive(Default, Component)]
struct ActionQueue(VecDeque<(Box<dyn Action>, ActionConfig)>);

#[derive(Default, Component)]
struct CurrentAction(Option<(Box<dyn Action>, ActionConfig)>);

struct ActionConfig {
    repeat: bool,
}

impl Into<ActionConfig> for AddConfig {
    fn into(self) -> ActionConfig {
        ActionConfig {
            repeat: self.repeat,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{world::*, *};

    struct EmptyAction;
    impl Action for EmptyAction {
        fn add(&mut self, _actor: Entity, _world: &mut World, _commands: &mut ActionCommands) {}
        fn remove(&mut self, _actor: Entity, _world: &mut World) {}
        fn stop(&mut self, _actor: Entity, _world: &mut World) {}
    }

    #[test]
    fn add() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world
            .action_builder(e, AddConfig::default())
            .add(EmptyAction)
            .add(EmptyAction)
            .add(EmptyAction)
            .apply();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 2);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 1);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn stop() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world.add_action(e, EmptyAction, AddConfig::default());

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.stop_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 1);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn clear() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world
            .action_builder(e, AddConfig::default())
            .add(EmptyAction)
            .add(EmptyAction)
            .add(EmptyAction)
            .add(EmptyAction)
            .add(EmptyAction)
            .apply();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 4);

        world.clear_actions(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }
}
