#![allow(clippy::needless_doctest_main)]
#![warn(missing_docs)]

/*!
# Bevy Sequential Actions

A [Bevy](https://bevyengine.org) library that aims to execute a queue of various actions in a sequential manner.
This generally means that one action runs at a time, and when it is done,
the next action will start and so on until the queue is empty.

## Getting Started

#### Plugin

In order for everything to work, the [`SequentialActionsPlugin`] must be added to your [`App`](bevy_app::App).

```rust,no_run
use bevy::prelude::*;
use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SequentialActionsPlugin::default())
        .run();
}
```

#### Modifying Actions

An action is anything that implements the [`Action`] trait, and can be added to any [`Entity`] that contains the [`ActionsBundle`].
An entity with actions is referred to as an `agent`.

```rust,no_run
# use bevy::prelude::*;
# use bevy_sequential_actions::*;
# use shared::actions::QuitAction;
#
fn setup(mut commands: Commands) {
#   let action_a = QuitAction;
#   let action_b = QuitAction;
#   let action_c = QuitAction;
#   let action_d = QuitAction;
#   let action_e = QuitAction;
#   let action_f = QuitAction;
#
    // Create entity with ActionsBundle
    let agent = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single action with default config
    commands.actions(agent).add(action_a);

    // Add multiple actions with custom config
    commands
        .actions(agent)
        .config(AddConfig {
            // Add each action to the back of the queue
            order: AddOrder::Back,
            // Start the next action if nothing is currently running
            start: true,
            // Repeat the action
            repeat: Repeat::None,
        })
        .add(action_b)
        .add(action_c);

    // Add a collection of actions that run in parallel.
    // This means that all actions will start and stop at the same time,
    // as the whole collection is treated as "one action".
    commands
        .actions(agent)
        .add_many(
            ExecutionMode::Parallel,
            actions![
                action_d,
                action_e,
                action_f,
            ]
        );
}
```

#### Implementing an Action

The [`Action`] trait contains two methods:

* The [`on_start`](Action::on_start) method which is called when an action is started.
* The [`on_stop`](Action::on_stop) method which is called when an action is stopped.

Every action is responsible for advancing the queue.
There are two ways of doing this:

* Using the [`ActionFinished`] component on an `agent`.
  By default, a system in [`CoreStage::Last`](bevy_app::CoreStage::Last) will advance the queue if all active actions are finished.
  This is the typical approach as it composes well with other actions running in parallel.
* Calling the [`next`](ModifyActions::next) method on an `agent`.
  This simply advances the queue at the end of the current stage it was called in.
  Useful for short one-at-a-time actions.

A simple wait action follows.

```rust,no_run
# use bevy::prelude::*;
# use bevy_sequential_actions::*;
#
pub struct WaitAction {
    duration: f32, // Seconds
    current: Option<f32>, // None
}

impl Action for WaitAction {
    fn on_start(&mut self, agent: Entity, world: &mut World, _commands: &mut ActionCommands) {
        // Take current duration (if paused), or use full duration
        let duration = self.current.take().unwrap_or(self.duration);

        // Run the wait system on the agent
        world.entity_mut(agent).insert(Wait(duration));
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        // Remove the wait component from the agent
        let wait = world.entity_mut(agent).remove::<Wait>();

        // Store current duration when paused
        if let StopReason::Paused = reason {
            self.current = Some(wait.unwrap().0);
        }
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait_system(mut wait_q: Query<(&mut Wait, &mut ActionFinished)>, time: Res<Time>) {
    for (mut wait, mut finished) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();

        // Confirm finished state every frame
        if wait.0 <= 0.0 {
            finished.confirm_and_reset();
        }
    }
}
```

#### Warning

One thing to keep in mind is that you should not modify actions using [`World`] inside the [`Action`] trait.
We cannot borrow a mutable action from an `agent` while also passing a mutable world to it.
And so, the action is detached from an `agent` when the trait methods are called.
Since an `agent` cannot hold an action while inside the [`Action`] trait,
the logic for advancing the action queue will not work properly.

This is why [`ActionCommands`] was created, so you can modify actions inside the [`Action`] trait in a deferred way.

```rust,no_run
# use bevy::{ecs::schedule::StateData, prelude::*};
# use bevy_sequential_actions::*;
#
pub struct SetStateAction<T: StateData>(T);

impl<T: StateData> Action for SetStateAction<T> {
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        world
            .resource_mut::<State<T>>()
            .set(self.0.clone())
            .unwrap();

        // Bad. The action queue will advance immediately.
        world.actions(agent).next();

        // Good. The action queue will advance a bit later.
        commands.actions(agent).next();

        // Also good. Does the same as above.
        commands.add(move |w: &mut World| {
            w.actions(agent).next();
        });

        // Also good. By default, The action queue will advance at the end of the frame.
        world.get_mut::<ActionFinished>(agent).unwrap().confirm_and_persist();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
```
*/

use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    slice::IterMut,
};

use bevy_ecs::prelude::*;

mod action_commands;
mod commands;
mod macros;
mod plugin;
mod traits;
mod world;

#[cfg(test)]
mod tests;

pub use action_commands::*;
pub use commands::*;
pub use macros::*;
pub use plugin::*;
pub use traits::*;
pub use world::*;

/// The component bundle that all entities with actions must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    finished: ActionFinished,
    queue: ActionQueue,
    current: CurrentAction,
}

/// Component for counting how many active actions have finished.
#[derive(Default, Component)]
pub struct ActionFinished {
    reset_count: u16,
    persist_count: u16,
}

impl ActionFinished {
    /// Confirms that an [`Action`] is finished by incrementing a counter.
    /// This should be called __every frame__,
    /// as the counter is reset in the [`Stage`] specified by [`SequentialActionsPlugin`].
    /// By default, the [`CoreStage::Last`](bevy_app::CoreStage::Last) is used.
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
            repeat: Repeat::None,
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
#[derive(Clone, Copy, Default)]
pub enum Repeat {
    /// Don't repeat the [`action`](Action).
    #[default]
    None,
    /// Repeat the [`action`](Action) by a specified amount.
    Amount(u32),
    /// Repeat the [`action`](Action) forever.
    Forever,
}

impl Repeat {
    fn process(&mut self) -> bool {
        match self {
            Repeat::None => false,
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
    /// Execute the [`actions`](Action) in sequence, i.e. one by one.
    Sequential,
    /// Execute the [`actions`](Action) in parallel, i.e. all at once.
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
