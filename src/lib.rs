#![warn(missing_docs, rustdoc::all)]

/*!
<div align="center">

# Bevy Sequential Actions

[![crates.io](https://img.shields.io/crates/v/bevy-sequential-actions?style=flat-square)](https://crates.io/crates/bevy-sequential-actions)
[![github.com](https://img.shields.io/github/stars/hikikones/bevy-sequential-actions?logo=github&style=flat-square)](https://github.com/hikikones/bevy-sequential-actions)
[![MIT/Apache 2.0](https://img.shields.io/crates/l/bevy-sequential-actions?style=flat-square)](https://github.com/hikikones/bevy-sequential-actions#license)

A [Bevy](https://bevyengine.org) library that aims to execute a queue of various actions in a sequential manner.
This generally means that one action runs at a time, and when it is done,
the next action will start and so on until the queue is empty.

</div>

## 📜 Getting Started

#### Plugin

The quickest way for getting started is adding the [`SequentialActionsPlugin`] to your [`App`].

```rust,no_run
# use bevy_ecs::prelude::*;
# use bevy_app::prelude::*;
use bevy_sequential_actions::*;
#
# struct DefaultPlugins;
# impl Plugin for DefaultPlugins { fn build(&self, _app: &mut App) {} }

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SequentialActionsPlugin))
        .run();
}
```

#### Implementing an Action

An action is anything that implements the [`Action`] trait.
The trait contains various methods that together defines the _lifecycle_ of an action.
From this, you can create any action that can last as long as you like,
and do as much as you like.

An entity with actions is referred to as an `agent`.

A simple wait action follows.

```rust,no_run
# use bevy_ecs::prelude::*;
# use bevy_sequential_actions::*;
#
# struct Time;
# impl Resource for Time {}
# impl Time { fn delta_seconds(&self) -> f32 { 0.0 } }
#
pub struct WaitAction {
    duration: f32, // Seconds
    current: Option<f32>, // None
}

impl Action for WaitAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        // Determine if wait timer has reached zero.
        // By default, this method is called every frame in the Last schedule.
        world.get::<WaitTimer>(agent).unwrap().0 <= 0.0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        // Take current time (if paused), or use full duration.
        let duration = self.current.take().unwrap_or(self.duration);

        // Run the wait timer system on the agent.
        world.entity_mut(agent).insert(WaitTimer(duration));

        // Is action already finished?
        // Returning true here will immediately advance the action queue.
        self.is_finished(agent, world)
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason) {
        // Do nothing if agent has been despawned.
        let Some(agent) = agent else { return };

        // Take the wait timer component from the agent.
        let wait_timer = world.entity_mut(agent).take::<WaitTimer>();

        // Store current time when paused.
        if reason == StopReason::Paused {
            self.current = Some(wait_timer.unwrap().0);
        }
    }
}

#[derive(Component)]
struct WaitTimer(f32);

fn wait_system(mut wait_timer_q: Query<&mut WaitTimer>, time: Res<Time>) {
    for mut wait_timer in &mut wait_timer_q {
        wait_timer.0 -= time.delta_seconds();
    }
}
```

#### Modifying Actions

Actions can be added to any [`Entity`] that contains the [`ActionsBundle`].
This is is done through the [`actions(agent)`](ActionsProxy::actions)
extension method implemented for both [`Commands`] and [`World`].
See the [`ModifyActions`] trait for available methods.

```rust,no_run
# use bevy_app::AppExit;
# use bevy_ecs::prelude::*;
# use bevy_sequential_actions::*;
#
# struct EmptyAction;
# impl Action for EmptyAction {
#   fn is_finished(&self, _a: Entity, _w: &World) -> bool { true }
#   fn on_start(&mut self, _a: Entity, _w: &mut World) -> bool { true }
#   fn on_stop(&mut self, _a: Option<Entity>, _w: &mut World, _r: StopReason) {}
# }
#
fn setup(mut commands: Commands) {
#   let action_a = EmptyAction;
#   let action_b = EmptyAction;
#   let action_c = EmptyAction;
#   let action_d = EmptyAction;
#
    // Spawn entity with the bundle
    let agent = commands.spawn(ActionsBundle::new()).id();
    commands
        .actions(agent)
        // Add a single action
        .add(action_a)
        // Add multiple actions
        .add_many(actions![
            action_b,
            action_c,
            action_d
        ])
        // Add an anonymous action with a closure
        .add(|_agent, world: &mut World| -> bool {
            // on_start
            world.send_event(AppExit::Success);
            true
        });
}
```
*/

use std::{collections::VecDeque, fmt::Debug};

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{component::ComponentId, prelude::*, query::QueryFilter, world::DeferredWorld};
use bevy_log::{debug, warn};

mod commands;
mod macros;
mod plugin;
mod traits;
mod world;

pub use commands::*;
pub use plugin::*;
pub use traits::*;
pub use world::*;

/// A boxed [`Action`].
pub type BoxedAction = Box<dyn Action>;

/// The component bundle that all entities with actions must have.
#[derive(Default, Bundle)]
pub struct ActionsBundle {
    current: CurrentAction,
    queue: ActionQueue,
}

impl ActionsBundle {
    /// Creates a new [`Bundle`] that contains the necessary components
    /// that all entities with actions must have.
    pub const fn new() -> Self {
        Self {
            current: CurrentAction(None),
            queue: ActionQueue(VecDeque::new()),
        }
    }

    /// Creates a new [`Bundle`] with specified `capacity` for the action queue.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            current: CurrentAction(None),
            queue: ActionQueue(VecDeque::with_capacity(capacity)),
        }
    }
}

/// The current action for an `agent`.
#[derive(Debug, Default, Component, Deref, DerefMut)]
pub struct CurrentAction(Option<BoxedAction>);

impl CurrentAction {
    /// The [`on_remove`](bevy_ecs::component::ComponentHooks::on_remove) component lifecycle hook
    /// used by [`SequentialActionsPlugin`] for cleaning up the current action when an `agent` is despawned.
    pub fn on_remove_hook(mut world: DeferredWorld, agent: Entity, _component_id: ComponentId) {
        let mut current_action = world.get_mut::<Self>(agent).unwrap();
        if let Some(mut action) = current_action.take() {
            world.commands().add(move |world: &mut World| {
                action.on_stop(None, world, StopReason::Canceled);
                action.on_remove(None, world);
                action.on_drop(None, world, DropReason::Done);
            });
        }
    }

    /// Observer for cleaning up the current action when an `agent` is despawned.
    pub fn on_remove_trigger<F: QueryFilter>(
        trigger: Trigger<OnRemove, Self>,
        mut query: Query<&mut Self, F>,
        mut commands: Commands,
    ) {
        let agent = trigger.entity();
        if let Ok(mut current_action) = query.get_mut(agent) {
            if let Some(mut action) = current_action.take() {
                commands.add(move |world: &mut World| {
                    action.on_stop(None, world, StopReason::Canceled);
                    action.on_remove(None, world);
                    action.on_drop(None, world, DropReason::Done);
                });
            }
        }
    }
}

/// The action queue for an `agent`.
#[derive(Debug, Default, Component, Deref, DerefMut)]
pub struct ActionQueue(VecDeque<BoxedAction>);

impl ActionQueue {
    /// The [`on_remove`](bevy_ecs::component::ComponentHooks::on_remove) component lifecycle hook
    /// used by [`SequentialActionsPlugin`] for cleaning up the action queue when an `agent` is despawned.
    pub fn on_remove_hook(mut world: DeferredWorld, agent: Entity, _component_id: ComponentId) {
        let mut action_queue = world.get_mut::<Self>(agent).unwrap();
        if !action_queue.is_empty() {
            let actions = std::mem::take(&mut action_queue.0);
            world.commands().add(move |world: &mut World| {
                for mut action in actions {
                    action.on_remove(None, world);
                    action.on_drop(None, world, DropReason::Cleared);
                }
            });
        }
    }

    /// Observer for cleaning up the action queue when an `agent` is despawned.
    pub fn on_remove_trigger<F: QueryFilter>(
        trigger: Trigger<OnRemove, Self>,
        mut query: Query<&mut Self, F>,
        mut commands: Commands,
    ) {
        let agent = trigger.entity();
        if let Ok(mut action_queue) = query.get_mut(agent) {
            if !action_queue.is_empty() {
                let actions = std::mem::take(&mut action_queue.0);
                commands.add(move |world: &mut World| {
                    for mut action in actions {
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Cleared);
                    }
                });
            }
        }
    }
}

/// Configuration for actions to be added.
#[derive(Debug, Clone, Copy)]
pub struct AddConfig {
    /// Start the next action in the queue if nothing is currently running.
    pub start: bool,
    /// The queue order for actions to be added.
    pub order: AddOrder,
}

impl AddConfig {
    /// Returns a new configuration for actions to be added.
    pub const fn new(start: bool, order: AddOrder) -> Self {
        Self { start, order }
    }
}

impl Default for AddConfig {
    fn default() -> Self {
        Self::new(true, AddOrder::Back)
    }
}

/// The queue order for actions to be added.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AddOrder {
    /// An action is added to the back of the queue.
    #[default]
    Back,
    /// An action is added to the front of the queue.
    Front,
}

/// The reason why an [`Action`] was stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The action was finished.
    Finished,
    /// The action was canceled.
    Canceled,
    /// The action was paused.
    Paused,
}

/// The reason why an [`Action`] was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    /// The action is considered done as it was either finished or canceled
    /// without being skipped or cleared from the action queue.
    Done,
    /// The action was skipped. This happens either deliberately,
    /// or because an action was added to an `agent` that does not exist or is missing the [`ActionsBundle`].
    Skipped,
    /// The action queue was cleared. This happens either deliberately,
    /// or because an `agent` was despawned.
    Cleared,
}
