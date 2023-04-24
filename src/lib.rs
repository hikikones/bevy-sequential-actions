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
# use bevy_ecs::prelude::*;
# use bevy_app::prelude::*;
use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugin(SequentialActionsPlugin::default())
        .run();
}
```

#### Modifying Actions

An action is anything that implements the [`Action`] trait,
and can be added to any [`Entity`] that contains the [`ActionsBundle`].
An entity with actions is referred to as an `agent`.
See the [`ModifyActions`] trait for available methods.

```rust,no_run
# use bevy_ecs::prelude::*;
# use bevy_sequential_actions::*;
#
# struct EmptyAction;
# impl Action for EmptyAction {
#   fn is_finished(&self, _a: Entity, _w: &World) -> bool { true }
#   fn on_start(&mut self, _a: Entity, _w: &mut World) {}
#   fn on_stop(&mut self, _a: Entity, _w: &mut World, _r: StopReason) {}
# }
#
fn setup(mut commands: Commands) {
#   let action_a = EmptyAction;
#   let action_b = EmptyAction;
#   let action_c = EmptyAction;
#   let action_d = EmptyAction;
#
    let agent = commands.spawn(ActionsBundle::default()).id();
    commands
        .actions(agent)
        .add(action_a)
        .add_many(actions![
            action_b,
            action_c
        ])
        .order(AddOrder::Front)
        .add(action_d)
        // ...
#       ;
}
```

#### Implementing an Action

The [`Action`] trait contains 3 required methods:

* The [`is_finished`](Action::is_finished) method to determine if an action is finished.
* The [`on_start`](Action::on_start) method which is called when an action is started.
* The [`on_stop`](Action::on_stop) method which is called when an action is stopped.

In addition, there are 2 optional methods:

* The [`on_add`](Action::on_add) method which is called when an action is added to the queue.
* The [`on_remove`](Action::on_remove) method which is called when an action is removed from the queue.

A simple countdown action follows.

```rust,no_run
# use bevy_ecs::prelude::*;
# use bevy_sequential_actions::*;
#
pub struct CountdownAction {
    count: i32,
    current: Option<i32>,
}

impl Action for CountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        // Determine if countdown has reached zero
        world.get::<Countdown>(agent).unwrap().0 <= 0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        // Take current count (if paused), or use full count
        let count = self.current.take().unwrap_or(self.count);

        // Run the countdown system on the agent
        world.entity_mut(agent).insert(Countdown(count));
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        // Take the countdown component from the agent
        let countdown = world.entity_mut(agent).take::<Countdown>();

        // Store current count when paused
        if let StopReason::Paused = reason {
            self.current = Some(countdown.unwrap().0);
        }
    }
}

#[derive(Component)]
struct Countdown(i32);

fn countdown_system(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 -= 1;
    }
}
```

#### Warning

One thing to keep in mind is when modifying actions using [`World`] inside the [`Action`] trait.
In order to pass a mutable reference to world when calling the trait methods,
the action has to be temporarily removed from an `agent`.
This means that depending on what you do,
the logic for advancing the action queue might not work properly.

In general, there are two rules when modifying actions inside the action trait:

* When adding new actions, the [`start`](ModifyActions::start) property should be set to `false`.
* The [`execute`](ModifyActions::execute) and [`next`](ModifyActions::next) methods should not be used.
*/

use std::collections::VecDeque;

use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;

mod commands;
mod macros;
mod plugin;
mod traits;
mod world;

pub use commands::*;
pub use macros::*;
pub use plugin::*;
pub use traits::*;
pub use world::*;

/// A boxed [`Action`].
pub type BoxedAction = Box<dyn Action>;

/// The component bundle that all entities with actions must have.
#[derive(Bundle)]
pub struct ActionsBundle<T: AgentMarker = DefaultAgentMarker> {
    marker: T,
    current: CurrentAction,
    queue: ActionQueue,
}

impl Default for ActionsBundle<DefaultAgentMarker> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: AgentMarker> ActionsBundle<T> {
    /// Creates a new [`Bundle`] that contains the necessary components
    /// that all entities with actions must have.
    pub fn new() -> Self {
        Self {
            marker: T::default(),
            current: CurrentAction(None),
            queue: ActionQueue(VecDeque::new()),
        }
    }
}

impl<I, T: AgentMarker> From<I> for ActionsBundle<T>
where
    I: IntoIterator<Item = BoxedAction>,
{
    fn from(actions: I) -> Self {
        Self {
            marker: T::default(),
            current: CurrentAction(None),
            queue: ActionQueue(VecDeque::from_iter(actions)),
        }
    }
}

/// The default marker component for agents.
/// Part of [`ActionsBundle::default`].
#[derive(Default, Component)]
pub struct DefaultAgentMarker;

/// The current action for an `agent`.
/// Part of [`ActionsBundle`].
///
/// Note that you are not supposed to use this directly.
#[derive(Component, Deref, DerefMut)]
pub struct CurrentAction(Option<BoxedAction>);

/// The action queue for an `agent`.
/// Part of [`ActionsBundle`].
///
/// Note that you are not supposed to use this directly.
#[derive(Component, Deref, DerefMut)]
pub struct ActionQueue(VecDeque<BoxedAction>);

/// The queue order for an [`Action`] to be added.
#[derive(Debug, Default, Clone, Copy)]
pub enum AddOrder {
    /// An action is added to the back of the queue.
    #[default]
    Back,
    /// An action is added to the front of the queue.
    Front,
}

/// The reason why an [`Action`] was stopped.
#[derive(Debug, Clone, Copy)]
pub enum StopReason {
    /// The action was finished.
    Finished,
    /// The action was canceled.
    Canceled,
    /// The action was paused.
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
