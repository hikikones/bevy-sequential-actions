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
        .add_plugin(SequentialActionsPlugin)
        .run();
}
```

#### Modifying Actions

An action is anything that implements the [`Action`] trait,
and can be added to any [`Entity`] that contains the [`ActionsBundle`].
An entity with actions is referred to as an `agent`.
See the [`ModifyActions`] trait for available methods.

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
#
    let agent = commands.spawn(ActionsBundle::new()).id();
    commands
        .actions(agent)
        .add(action_a)
        .add_parallel(actions![
            action_b,
            action_c
        ])
        .repeat(Repeat::Forever)
        .order(AddOrder::Back)
        .add(action_d)
        // ...
#       ;
}
```

#### Implementing an Action

The [`Action`] trait contains two methods:

* The [`on_start`](Action::on_start) method which is called when an action is started.
* The [`on_stop`](Action::on_stop) method which is called when an action is stopped.

In order for the action queue to advance, every action has to somehow signal when they are finished.
There are two ways of doing this:

* Using the [`ActionFinished`] component on an `agent`.
  By default, a system at the end of the frame will advance the queue if all active actions are finished.
  This is the typical approach as it composes well with other actions running in parallel.
* Calling the [`next`](ModifyActions::next) method on an `agent`.
  This simply advances the queue, and is useful for short one-at-a-time actions.

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
    fn on_start(&mut self, agent: Entity, world: &mut World) {
        // Take current duration (if paused), or use full duration
        let duration = self.current.take().unwrap_or(self.duration);

        // Run the wait system on the agent
        world.entity_mut(agent).insert(Wait(duration));
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        // Remove the wait component from the agent
        let wait = world.entity_mut(agent).take::<Wait>();

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

One thing to keep in mind is when modifying actions using [`World`] inside the [`Action`] trait.
We cannot borrow a mutable action from an `agent` while also passing a mutable world to it.
Since an action is detached from an `agent` when the trait methods are called,
the logic for advancing the action queue will not work properly.

Use the [`deferred_actions`](DeferredActionsProxy::deferred_actions) method for deferred world mutation.

```rust,no_run
# use bevy::{ecs::schedule::States, prelude::*};
# use bevy_sequential_actions::*;
#
pub struct SetStateAction<S: States>(S);

impl<S: States> Action for SetStateAction<S> {
    fn on_start(&mut self, agent: Entity, world: &mut World) {
        // Set state
        world.resource_mut::<NextState<S>>().set(self.0.clone());

        // Bad. The action queue will advance immediately.
        world.actions(agent).next();

        // Good. The action queue will advance a bit later.
        world.deferred_actions(agent).next();

        // Also good. Does the same as above.
        world.deferred_actions(agent).custom(move |w: &mut World| {
            w.actions(agent).next();
        });

        // Also good. By default, the action queue will advance at the end of the frame.
        world.get_mut::<ActionFinished>(agent).unwrap().confirm_and_persist();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
```
*/

use std::collections::VecDeque;

use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;

mod commands;
mod macros;
mod plugin;
mod traits;
mod world;

// #[cfg(test)]
// mod tests;

pub use commands::*;
pub use macros::*;
pub use plugin::*;
pub use traits::*;
pub use world::*;

/// A boxed action.
pub type BoxedAction = Box<dyn Action>;

/// The component bundle that all entities with actions must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    marker: ActionMarker,
    current: CurrentAction,
    queue: ActionQueue,
}

impl ActionsBundle {
    /// Creates a new [`Bundle`] that contains the necessary components
    /// that all entities with actions must have.
    pub const fn new() -> Self {
        Self {
            marker: ActionMarker,
            current: CurrentAction(None),
            queue: ActionQueue(VecDeque::new()),
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

#[derive(Clone, Copy)]
struct AddConfig {
    start: bool,
    order: AddOrder,
}

impl AddConfig {
    const fn new() -> Self {
        Self {
            start: true,
            order: AddOrder::Back,
        }
    }
}

#[derive(Default, Component)]
struct ActionMarker;

#[derive(Default, Component, Deref, DerefMut)]
struct CurrentAction(Option<BoxedAction>);

#[derive(Default, Component, Deref, DerefMut)]
struct ActionQueue(VecDeque<BoxedAction>);

impl ActionQueue {
    fn push(&mut self, order: AddOrder, action: BoxedAction) {
        match order {
            AddOrder::Back => self.0.push_back(action),
            AddOrder::Front => self.0.push_front(action),
        }
    }
}

// #[derive(Component)]
// struct ActionAgent(Entity);

// #[derive(Component, Deref)]
// struct ActionWrapper(BoxedAction);

// #[derive(Bundle)]
// struct ActionBundle {
//     agent: ActionAgent,
//     action: ActionWrapper,
// }
