#![warn(missing_docs)]

//! # Bevy Sequential Actions
//!
//! `bevy_sequential_actions` is a library for the [Bevy game engine](https://bevyengine.org/ "bevy game engine")
//! that aims to execute a list of actions in a sequential manner. This generally means that one action runs at a time,
//! and when it is done, the next action will start, and so on until the list is empty.
//!
//! ## Getting Started
//!
//! An action is anything that implements the [`Action`] trait,
//! and can be added to any [`Entity`] that contains the [`ActionsBundle`].
//! Each action must signal when they are finished,
//! which is done by calling the [`next`](ModifyActionsExt::next) method.
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_sequential_actions::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(MinimalPlugins)
//!         .add_startup_system(setup)
//!         .add_system(wait)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // Create entity with ActionsBundle
//!     let entity = commands.spawn_bundle(ActionsBundle::default()).id();
//!
//!     // Add a single action with default config
//!     commands.action(entity).add(WaitAction(1.0));
//!
//!     // Add multiple actions with custom config
//!     commands
//!         .action(entity)
//!         .config(AddConfig {
//!             // Add each action to the back of the queue
//!             order: AddOrder::Back,
//!             // Start action if nothing is currently running
//!             start: false,
//!             // Repeat the action
//!             repeat: false,
//!         })
//!         .add(WaitAction(2.0))
//!         .add(WaitAction(3.0));
//! }
//!
//! struct WaitAction(f32);
//!
//! impl Action for WaitAction {
//!     fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
//!         world.entity_mut(entity).insert(Wait(self.0));
//!     }
//!
//!     fn remove(&mut self, entity: Entity, world: &mut World) {
//!         world.entity_mut(entity).remove::<Wait>();
//!     }
//!
//!     fn stop(&mut self, entity: Entity, world: &mut World) {
//!         self.remove(entity, world);
//!     }
//! }
//!
//! #[derive(Component)]
//! struct Wait(f32);
//!
//! fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
//!     for (entity, mut wait) in wait_q.iter_mut() {
//!         wait.0 -= time.delta_seconds();
//!         if wait.0 <= 0.0 {
//!             // Action is finished, issue next.
//!             commands.action(entity).next();
//!         }
//!     }
//! }
//! ```

use std::collections::VecDeque;

use bevy_ecs::prelude::*;

mod action_commands;
mod commands;
mod traits;
mod world;

pub use action_commands::*;
pub use commands::*;
pub use traits::*;
pub use world::*;

/// The component bundle that all entities with actions must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    queue: ActionQueue,
    current: CurrentAction,
}

/// The queue order for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub enum AddOrder {
    /// An [`action`](Action) is added to the **back** of the queue.
    Back,
    /// An [`action`](Action) is added to the **front** of the queue.
    Front,
}

/// Configuration for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub struct AddConfig {
    /// Specify the queue order for the [`action`](Action) to be added.
    pub order: AddOrder,
    /// Start the [`action`](Action) if nothing is currently running.
    pub start: bool,
    /// Repeat the [`action`](Action) when it has finished. This is done by adding it back to the queue when it is removed.
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

#[allow(clippy::from_over_into)]
impl Into<ActionConfig> for AddConfig {
    fn into(self) -> ActionConfig {
        ActionConfig {
            repeat: self.repeat,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    struct EmptyAction;
    impl Action for EmptyAction {
        fn start(&mut self, _entity: Entity, _world: &mut World, _commands: &mut ActionCommands) {}
        fn remove(&mut self, _entity: Entity, _world: &mut World) {}
        fn stop(&mut self, _entity: Entity, _world: &mut World) {}
    }

    #[test]
    fn add() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world
            .action(e)
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .submit();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 2);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 1);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn stop() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world.action(e).add(EmptyAction);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.action(e).stop();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 1);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn clear() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world
            .action(e)
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .submit();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 4);

        world.action(e).clear();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn repeat() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world
            .action(e)
            .config(AddConfig {
                order: AddOrder::Back,
                start: true,
                repeat: true,
            })
            .add(EmptyAction);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.action(e).next();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn despawn() {
        struct DespawnAction;
        impl Action for DespawnAction {
            fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
                world.despawn(entity);
            }
            fn remove(&mut self, _entity: Entity, _world: &mut World) {}
            fn stop(&mut self, _entity: Entity, _world: &mut World) {}
        }

        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world.action(e).add(DespawnAction);

        assert!(world.get_entity(e).is_none());
    }
}
