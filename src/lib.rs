#![warn(missing_docs)]

//! # Bevy Sequential Actions
//!
//! `bevy-sequential-actions` is a library for the [Bevy](https://bevyengine.org) game engine
//! that aims to execute a list of actions in a sequential manner.
//! This generally means that one action runs at a time, and when it is done,
//! the next action will start and so on until the list is empty.
//!
//! ## Getting Started
//!
//! An action is anything that implements the [`Action`] trait,
//! and can be added to any [`Entity`] that contains the [`ActionsBundle`].
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_sequential_actions::*;
//! # fn main() {}
//! # struct EmptyAction;
//! # impl Action for EmptyAction {
//! #     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {}
//! #     fn finish(&mut self, entity: Entity, world: &mut World) {}
//! # }
//! fn setup(mut commands: Commands) {
//! #   let wait_action = EmptyAction;
//! #   let move_action = EmptyAction;
//! #   let quit_action = EmptyAction;
//! #
//!     // Create entity with ActionsBundle
//!     let entity = commands.spawn_bundle(ActionsBundle::default()).id();
//!     
//!     // Add a single action with default config
//!     commands.actions(entity).add(wait_action);
//!     
//!     // Add multiple actions with custom config
//!     commands
//!         .actions(entity)
//!         .config(AddConfig {
//!             // Add each action to the back of the queue
//!             order: AddOrder::Back,
//!             // Start action if nothing is currently running
//!             start: true,
//!             // Repeat the action
//!             repeat: false,
//!         })
//!         .add(move_action)
//!         .add(quit_action);
//! }
//! ```

use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use bevy_ecs::prelude::*;

mod action_commands;
mod commands;
mod traits;
mod world;

#[cfg(test)]
mod tests;

pub use action_commands::*;
pub use commands::*;
pub use traits::*;
pub use world::*;

/// The component bundle that all entities with [`actions`](Action) must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    queue: ActionQueue,
    current: CurrentAction,
}

/// The queue order for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub enum AddOrder {
    /// An [`action`](Action) is added to the back of the queue.
    Back,
    /// An [`action`](Action) is added to the front of the queue.
    Front,
}

/// Configuration for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub struct AddConfig {
    /// Specify the queue order for the [`action`](Action) to be added.
    pub order: AddOrder,
    /// Start the [`action`](Action) if nothing is currently running.
    pub start: bool,
    /// Repeat the [`action`](Action) when it has finished.
    /// This is done by adding it back to the queue when it is removed.
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

type ActionTuple = (Box<dyn Action>, ActionConfig);

#[derive(Default, Component)]
struct ActionQueue(VecDeque<ActionTuple>);

#[derive(Default, Component)]
struct CurrentAction(Option<ActionTuple>);

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

impl Deref for ActionQueue {
    type Target = VecDeque<ActionTuple>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for CurrentAction {
    type Target = Option<ActionTuple>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActionQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DerefMut for CurrentAction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
