#![warn(missing_docs)]

//! # Bevy Sequential Actions
//!
//! A [Bevy](https://bevyengine.org) library
//! that aims to execute a list of various actions in a sequential manner.
//! This generally means that one action runs at a time, and when it is done,
//! the next action will start and so on until the list is empty.
//!
//! ## Getting Started
//!
//! An action is anything that implements the [`Action`] trait,
//! and can be added to any [`Entity`] that contains the [`ActionsBundle`].
//! An entity with actions is referred to as an `agent`.
//!
//! ```rust,no_run
//! # use bevy::prelude::*;
//! # use bevy_sequential_actions::*;
//! # use shared::actions::QuitAction;
//! #
//! fn setup(mut commands: Commands) {
//! #   let wait_action = QuitAction;
//! #   let move_action = QuitAction;
//! #   let quit_action = QuitAction;
//! #
//!     // Create entity with ActionsBundle
//!     let agent = commands.spawn_bundle(ActionsBundle::default()).id();
//!     
//!     // Add a single action with default config
//!     commands.actions(agent).add(wait_action);
//!     
//!     // Add multiple actions with custom config
//!     commands
//!         .actions(agent)
//!         .config(AddConfig {
//!             // Add each action to the back of the queue
//!             order: AddOrder::Back,
//!             // Start the next action if nothing is currently running
//!             start: true,
//!             // Repeat the action
//!             repeat: Repeat::Amount(0),
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
mod plugin;
mod traits;
mod world;

#[cfg(test)]
mod tests;

pub use action_commands::*;
pub use commands::*;
pub use plugin::*;
pub use traits::*;
pub use world::*;

/// The component bundle that all entities with actions must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    marker: ActionMarker,
    queue: ActionQueue,
    current: CurrentAction,
    count: FinishedCount,
}

/// Marker component for entities with [`ActionsBundle`].
#[derive(Default, Component)]
pub struct ActionMarker;

#[derive(Component)]
pub struct ActionFinished(bool);

impl ActionFinished {
    pub fn set(&mut self, v: bool) {
        self.0 = v;
    }
}

#[derive(Component)]
pub struct ActionAgent(Entity);

impl ActionAgent {
    pub fn id(&self) -> Entity {
        self.0
    }
}

#[derive(Default, Component)]
struct FinishedCount(u32);

/// Configuration for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub struct AddConfig {
    /// Specify the queue order for the [`action`](Action) to be added.
    pub order: AddOrder,
    /// Start the next [`action`](Action) in the queue if nothing is currently running.
    pub start: bool,
    /// Specify how many times the [`action`](Action) should be repeated.
    pub repeat: Repeat,
}

impl Default for AddConfig {
    fn default() -> Self {
        Self {
            order: AddOrder::Back,
            start: true,
            repeat: Repeat::Amount(0),
        }
    }
}

/// The queue order for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub enum AddOrder {
    /// An [`action`](Action) is added to the back of the queue.
    Back,
    /// An [`action`](Action) is added to the front of the queue.
    Front,
}

/// The repeat configuration for an [`Action`] to be added.
#[derive(Clone, Copy)]
pub enum Repeat {
    /// Repeat the [`action`](Action) `n` times.
    Amount(u32),
    /// Repeat the [`action`](Action) forever.
    Forever,
}

/// The reason why an [`Action`] was stopped.
#[derive(Clone, Copy)]
pub enum StopReason {
    /// The [`action`](Action) was finished.
    Finished,
    /// The [`action`](Action) was canceled.
    Canceled,
    /// The [`action`](Action) was paused.
    Paused,
}

/// The execution mode for a collection of [`actions`](Action) to be added.
pub enum ExecutionMode {
    /// Execute the [`actions`](Action) in sequence.
    Sequential,
    /// Execute the [`actions`](Action) in parallel.
    Parallel,
}

pub struct ActionIds(Entity, Entity);

impl ActionIds {
    pub fn agent(&self) -> Entity {
        self.0
    }

    pub fn status(&self) -> Entity {
        self.1
    }
}

/// A boxed [`Action`].
pub type BoxedAction = Box<dyn Action>;

type ActionTuple = (ActionType, ActionState);
type ActionPair = (BoxedAction, Option<Entity>);

#[derive(Default, Component)]
struct ActionQueue(VecDeque<ActionTuple>);

#[derive(Default, Component)]
struct CurrentAction(Option<ActionTuple>);

enum ActionType {
    Single(ActionPair),
    Multiple(Box<[ActionPair]>),
}

struct ActionState {
    repeat: Repeat,
}

impl From<AddConfig> for ActionState {
    fn from(cfg: AddConfig) -> Self {
        Self { repeat: cfg.repeat }
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
