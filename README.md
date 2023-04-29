<div align="center">

# Bevy Sequential Actions

[![crates.io](https://img.shields.io/crates/v/bevy-sequential-actions?style=flat-square)](https://crates.io/crates/bevy-sequential-actions)
[![docs.rs](https://img.shields.io/docsrs/bevy-sequential-actions?style=flat-square)](https://docs.rs/bevy_sequential_actions)
[![MIT/Apache 2.0](https://img.shields.io/crates/l/bevy-sequential-actions?style=flat-square)](https://github.com/hikikones/bevy-sequential-actions#license)

</div>

A [Bevy](https://bevyengine.org) library that aims to execute a queue of various actions in a sequential manner.
This generally means that one action runs at a time, and when it is done,
the next action will start and so on until the queue is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

## :scroll: Getting Started

#### Plugin

In order for everything to work, the `SequentialActionsPlugin` must be added to your `App`.

```rust
use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugin(SequentialActionsPlugin::default())
        .run();
}
```

#### Modifying Actions

An action is anything that implements the `Action` trait,
and can be added to any `Entity` that contains the `ActionsBundle`.
An entity with actions is referred to as an `agent`.
See the `ModifyActions` trait for available methods.

```rust
fn setup(mut commands: Commands) {
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
}
```

#### Implementing an Action

The `Action` trait contains 3 required methods:

* `is_finished` to determine if an action is finished or not.
    By default, this method is called every frame in `CoreSet::Last`.
* `on_start` which is called when an action is started.
* `on_stop` which is called when an action is stopped.

In addition, there are 3 optional methods:

* `on_add` which is called when an action is added to the queue.
* `on_remove` which is called when an action is removed from the queue.
* `on_drop` which is the last method to be called with full ownership.

A simple countdown action follows.

```rust
pub struct CountdownAction {
    count: i32,
    current: Option<i32>,
}

impl Action for CountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        // Determine if countdown has reached zero
        world.get::<Countdown>(agent).unwrap().0 <= 0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        // Take current count (if paused), or use full count
        let count = self.current.take().unwrap_or(self.count);

        // Run the countdown system on the agent
        world.entity_mut(agent).insert(Countdown(count));

        // Is action already finished?
        self.is_finished(agent, world)
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

#### :warning: Warning

One thing to keep in mind is when modifying actions using `World` inside the `Action` trait.
In order to pass a mutable reference to world when calling the trait methods,
the action has to be temporarily removed from an `agent`.
This means that depending on what you do,
the logic for advancing the action queue might not work properly.

In general, there are two rules when modifying actions for an `agent` inside the action trait:

* When adding new actions, you should either set the `start` property to `false`,
    or use the `ActionQueue` component directly.
* The `execute` and `next` methods should not be used.

## :paperclip: Examples

See the [examples](examples/) for more usage.
Each example can be run with `cargo run --example <example>`.

| Example | Description |
| ------- | ----------- |
| `basic` | Shows the basic usage of the library. |
| `pause` | Shows how to pause and resume an action. |
| `repeat` | Shows how to create an action that repeats. |
| `despawn` | Shows how to properly despawn an `agent`. |
| `parallel` | Shows how to create actions that run in parallel. |
| `schedule` | Shows how to use the plugin with two different schedules. |

## :pushpin: Compatibility

| bevy | bevy-sequential-actions |
| ---- | ----------------------- |
| 0.10 | 0.7                     |
| 0.9  | 0.6                     |
| 0.8  | 0.3 — 0.5               |
| 0.7  | 0.1 — 0.2               |

## :bookmark: License

`bevy-sequential-actions` is dual-licensed under either

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.