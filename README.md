<div align="center">

# Bevy Sequential Actions

[![crates.io](https://img.shields.io/crates/v/bevy-sequential-actions?style=flat-square)](https://crates.io/crates/bevy-sequential-actions)
[![docs.rs](https://img.shields.io/docsrs/bevy-sequential-actions?style=flat-square)](https://docs.rs/bevy_sequential_actions)
[![MIT/Apache 2.0](https://img.shields.io/crates/l/bevy-sequential-actions?style=flat-square)](https://github.com/hikikones/bevy-sequential-actions#license)


A [Bevy](https://bevyengine.org) library that aims to execute a queue of various actions in a sequential manner.
This generally means that one action runs at a time, and when it is done,
the next action will start and so on until the queue is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

</div>

## 📜 Getting Started

#### Plugin

The quickest way for getting started is adding the `SequentialActionsPlugin` to your `App`.

```rust
use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SequentialActionsPlugin))
        .run();
}
```

#### Implementing an Action

An action is anything that implements the `Action` trait.
The trait contains 3 required methods:

* `is_finished` to determine if an action is finished or not.
* `on_start` which is called when an action is started.
* `on_stop` which is called when an action is stopped.

In addition, there are 3 optional methods:

* `on_add` which is called when an action is added to the queue.
* `on_remove` which is called when an action is removed from the queue.
* `on_drop` which is the last method to be called with full ownership.

An entity with actions is referred to as an `agent`.

A simple wait action follows.

```rust
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

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
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

Actions can be added to any `Entity` that contains the `ActionsBundle`.
See the `ModifyActionsExt` trait for available methods.
The extension trait is implemented for both `EntityCommands` and `EntityWorldMut`.

```rust
fn setup(mut commands: Commands) {
    commands
        // Spawn entity with the bundle
        .spawn(ActionsBundle::new())
        // Add a single action
        .add_action(
            AddConfig {
                start: true, // Start next action if nothing is currently running
                order: AddOrder::Back, // Add the action to the back of the queue
            },
            action_a,
        )
        // Add multiple actions
        .add_actions(
            AddConfig::default(),
            actions![
                action_b,
                action_c
            ],
        )
        // Add an anonymous action with a closure
        .add_action(
            AddConfig::default(),
            |_agent, world: &mut World| -> bool {
                // on_start
                true
            },
        );
}
```

#### ⚠️ Warning

One thing to keep in mind is when modifying actions using `World` inside the `Action` trait.
In order to pass a mutable reference to world when calling the trait methods,
the action has to be temporarily removed from an `agent`.
This means that depending on what you do,
the logic for advancing the action queue might not work properly.

In general, there are two rules when modifying actions for an `agent` inside the action trait:

* When adding new actions, you should either set the `start` property in `AddConfig` to `false`,
    or push to the `ActionQueue` component directly.
* The `execute_actions` and `next_action` methods should not be used.

## 📎 Examples

See the [examples](examples/) for more usage.
Each example can be run with `cargo run --example <example>`.

| Example | Description |
| ------- | ----------- |
| `basic` | Shows the basic usage of the library. |
| `pause` | Shows how to pause and resume an action. |
| `repeat` | Shows how to create an action that repeats. |
| `parallel` | Shows how to create actions that run in parallel. |
| `sequence` | Shows how to create an action with a sequence of actions. |
| `schedule` | Shows how to use the plugin with two different schedules. |
| `custom` | Shows how to use a custom system for advancing the action queue. |
| `despawn` | Shows how to properly despawn an `agent`. |

## 📌 Compatibility

| bevy | bevy-sequential-actions |
| ---- | ----------------------- |
| 0.14 | 0.11                    |
| 0.13 | 0.10                    |
| 0.12 | 0.9                     |
| 0.11 | 0.8                     |
| 0.10 | 0.7                     |
| 0.9  | 0.6                     |
| 0.8  | 0.3 – 0.5               |
| 0.7  | 0.1 – 0.2               |

## 🔖 License

`bevy-sequential-actions` is dual-licensed under either

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.