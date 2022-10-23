#![warn(missing_docs)]

/*!

# Bevy Sequential Actions

A [Bevy](https://bevyengine.org) library

todo

*/

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
    slice::IterMut,
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
    finished: ActionFinished,
    queue: ActionQueue,
    current: CurrentAction,
}

/// Marker component for entities with actions.
#[derive(Default, Component)]
pub struct ActionMarker;

/// Component for counting how many active actions have finished.
#[derive(Default, Component)]
pub struct ActionFinished {
    reset_count: u16,
    persist_count: u16,
}

impl ActionFinished {
    /// Confirms that an [`Action`] is finished by incrementing a counter.
    /// This should be called __every frame__,
    /// as the counter is reset at the end of the frame.
    pub fn confirm_and_reset(&mut self) {
        self.reset_count += 1;
    }

    /// Confirms that an [`Action`] is finished by incrementing a counter.
    /// This should be called __only once__,
    /// as the counter will only reset when an active [`Action`] is [`stopped`](Action::on_stop).
    pub fn confirm_and_persist(&mut self) {
        self.persist_count += 1;
    }

    fn total(&self) -> u32 {
        self.reset_count as u32 + self.persist_count as u32
    }
}

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

impl Repeat {
    fn process(&mut self) -> bool {
        match self {
            Repeat::Amount(n) => {
                if *n == 0 {
                    return false;
                }
                *n -= 1;
                true
            }
            Repeat::Forever => true,
        }
    }
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

/// A boxed [`Action`].
pub type BoxedAction = Box<dyn Action>;

type ActionTuple = (ActionType, Repeat);

#[derive(Default, Component)]
struct ActionQueue(VecDeque<ActionTuple>);

#[derive(Default, Component)]
struct CurrentAction(Option<ActionTuple>);

enum ActionType {
    One([BoxedAction; 1]),
    Many(Box<[BoxedAction]>),
}

impl ActionType {
    fn iter_mut(&mut self) -> IterMut<BoxedAction> {
        match self {
            ActionType::One(a) => a.iter_mut(),
            ActionType::Many(a) => a.iter_mut(),
        }
    }

    fn len(&self) -> u32 {
        match self {
            ActionType::One(_) => 1,
            ActionType::Many(a) => a.len() as u32,
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
