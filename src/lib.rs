#![deny(missing_docs)]

//! # Bevy Sequential Actions
//!
//! `bevy_sequential_actions` is a library for the [Bevy game engine](https://bevyengine.org/ "bevy game engine")
//! that aims to execute a list of actions in a sequential manner. This generally means that one action runs at a time,
//! and when it is done, the next action will start, and so on until the list is empty.
//!
//! # Getting Started
//!
//! An action is anything that implements the [`Action`] trait, and can be added to any [`Entity`] that contains the
//! [`ActionsBundle`]. Each action must signal when they are finished, which is done by calling `next_action` on either
//! [`Commands`] or [`ActionCommands`].
//!
//! ```rust
//! use bevy::prelude::*;
//!
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
//!     let id = commands.spawn_bundle(ActionsBundle::default()).id();
//!
//!     // Add a single action with default config
//!     commands.add_action(id, WaitAction(1.0), AddConfig::default());
//!
//!     // Add multiple actions with custom config
//!     commands
//!         .action_builder(
//!             id,
//!             AddConfig {
//!                 // Add each action to the back of the queue
//!                 order: AddOrder::Back,
//!                 // Start action if nothing is currently running
//!                 start: false,
//!                 // Repeat the action         
//!                 repeat: false,
//!             },
//!         )
//!         .push(WaitAction(2.0))
//!         .push(WaitAction(3.0))
//!         .submit();
//! }
//!
//! struct WaitAction(f32);
//!
//! impl Action for WaitAction {
//!     fn add(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
//!         world.entity_mut(actor).insert(Wait(self.0));
//!     }
//!
//!     fn remove(&mut self, actor: Entity, world: &mut World) {
//!         world.entity_mut(actor).remove::<Wait>();
//!     }
//!
//!     fn stop(&mut self, actor: Entity, world: &mut World) {
//!         self.remove(actor, world);
//!     }
//! }
//!
//! #[derive(Component)]
//! struct Wait(f32);
//!
//! fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
//!     for (actor, mut wait) in wait_q.iter_mut() {
//!         wait.0 -= time.delta_seconds();
//!         if wait.0 <= 0.0 {
//!             // Action is finished, issue next.
//!             commands.next_action(actor);
//!         }
//!     }
//! }
//! ```

use std::collections::VecDeque;

use bevy_ecs::prelude::*;

mod action_commands;
mod commands;
mod traits;

/// Contains the implementation for scheduling actions.
///
/// The `world` module is not exported by default because of potential misuse.
/// Typically one should use [`Commands`] and [`ActionCommands`] when modifying actions.
///
/// See warning further below.
///
/// # Example
///
/// ```rust
/// use bevy_sequential_actions::{*, world::*};
///
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
///         commands.next_action(actor);
///     }
///
///     fn remove(&mut self, actor: Entity, world: &mut World) {}
///     fn stop(&mut self, actor: Entity, world: &mut World) {}
/// }
///
/// fn exclusive_world(world: &mut World) {
///     let id = world.spawn().insert_bundle(ActionsBundle::default()).id();
///     world.add_action(id, EmptyAction, AddConfig::default());
/// }
/// ```
///
/// # Warning
///
/// Should only be used when working exclusively within a [`World`].
/// Using the world extension methods **inside** the implementation of an [`Action`] is **not** intended to work.
///
/// Here is an example of what not to do:
///
/// ```rust
/// use bevy_sequential_actions::{*, world::*};
///
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // By using `world` to issue next action, the change happens immediately
///         // and the current action will not be set.
///         // See [world.rs] for implementation details.
///         // Since current action is not set, the `remove` method will never be called.
///         world.next_action(actor); // <- bad
///
///         // You should always use the passed `commands` for issuing commands,
///         // as they are put in a queue and applied at the end. This ensures that
///         // the current action is set each time.
///         commands.next_action(actor); // <- good
///     }
///
///     fn remove(&mut self, actor: Entity, world: &mut World) {}
///     fn stop(&mut self, actor: Entity, world: &mut World) {}
/// }
/// ```
pub mod world;

pub use action_commands::*;
pub use commands::*;
pub use traits::*;

/// The trait that all actions must implement.
///
/// # Example
///
/// An empty action that does nothing.
/// All actions must declare when they are done.
/// This is done by calling [`ActionCommands::next_action`].
///
/// ```rust
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Action is finished, issue next.
///         commands.next_action(actor);
///     }
///
///     fn remove(&mut self, actor: Entity, world: &mut World) {}
///     fn stop(&mut self, actor: Entity, world: &mut World) {}
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an [`Action`] is started.
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands);
    /// The method that is called when an [`Action`] is removed.
    fn remove(&mut self, actor: Entity, world: &mut World);
    /// The method that is called when an [`Action`] is stopped.
    fn stop(&mut self, actor: Entity, world: &mut World);
}

/// The component bundle that all entities with actions must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    queue: ActionQueue,
    current: CurrentAction,
}

/// The order for an added [`Action`].
#[derive(Clone, Copy)]
pub enum AddOrder {
    /// An [`Action`] is added to the **back** of the queue.
    Back,
    /// An [`Action`] is added to the **front** of the queue.
    Front,
}

/// Configuration for the [`Action`] to be added.
#[derive(Clone, Copy)]
pub struct AddConfig {
    /// Specify the [`AddOrder`] of the [`Action`]. Either to the back of the queue, or to the front.
    pub order: AddOrder,
    /// Start the [`Action`] if nothing is currently running.
    pub start: bool,
    /// Repeat the [`Action`] when it has finished. This is done by adding it back to the queue when it is removed.
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
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
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
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .push(EmptyAction)
            .apply();

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 4);

        world.clear_actions(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_none());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }

    #[test]
    fn repeat() {
        let mut world = World::new();

        let e = world.spawn().insert_bundle(ActionsBundle::default()).id();

        world.add_action(
            e,
            EmptyAction,
            AddConfig {
                order: AddOrder::Back,
                start: true,
                repeat: true,
            },
        );

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);

        world.next_action(e);

        assert!(world.get::<CurrentAction>(e).unwrap().0.is_some());
        assert!(world.get::<ActionQueue>(e).unwrap().0.len() == 0);
    }
}
