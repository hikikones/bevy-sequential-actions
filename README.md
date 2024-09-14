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
The trait contains various methods that together defines the _lifecycle_ of an action.
From this, you can create any action that can last as long as you like,
and do as much as you like.

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

Actions can be added to any `Entity` that contains the `ActionsBundle`.
This is is done through the `actions(agent)`
extension method implemented for both `Commands` and `World`.
See the `ModifyActions` trait for available methods.

```rust
fn setup(mut commands: Commands) {
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

## 📎 Examples

See the [examples](examples/) for more usage.
Each example can be run with `cargo run --example <example>`.

| Example | Description |
| ------- | ----------- |
| `basic` | Basic usage of the library. |
| `pause` | Pause and resume an action. |
| `repeat` | Create an action that repeats. |
| `parallel` | Create actions that run in parallel. |
| `sequence` | Create action with a sequence of actions. |
| `custom` | Custom plugin with different schedules and action queue advancement. |

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