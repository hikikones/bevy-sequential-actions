<div align="center">

# Bevy Sequential Actions

[![crates.io](https://img.shields.io/crates/v/bevy-sequential-actions?style=flat-square)](https://crates.io/crates/bevy-sequential-actions)
[![docs.rs](https://img.shields.io/docsrs/bevy-sequential-actions?style=flat-square)](https://docs.rs/bevy_sequential_actions)
[![MIT/Apache 2.0](https://img.shields.io/crates/l/bevy-sequential-actions?style=flat-square)](https://github.com/hikikones/bevy-sequential-actions#license)


A simple library for managing and sequencing various actions in [Bevy](https://bevyengine.org).

<figure>
    <img src="https://github.com/user-attachments/assets/66b5b15e-96af-47bd-9371-eee8809d1294"/>
    <p><em>An entity with a queue of repeating actions</em></p>
</figure>

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
    // By default, this method is called every frame in the Last schedule.
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        // Determine if wait timer has reached zero.
        world.get::<WaitTimer>(agent).unwrap().0 <= 0.0
    }

    // This method is called when an action is started.
    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        // Take current time (if paused), or use full duration.
        let duration = self.current.take().unwrap_or(self.duration);

        // Run the wait timer system on the agent.
        world.entity_mut(agent).insert(WaitTimer(duration));

        // Is action already finished?
        // Returning true here will immediately advance the action queue.
        self.is_finished(agent, world)
    }

    // This method is called when an action is stopped.
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

    // Optional. This method is called when an action is added to the queue.
    fn on_add(&mut self, agent: Entity, world: &mut World) {}

    // Optional. This method is called when an action is removed from the queue.
    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {}

    // Optional. The last method that is called with full ownership.
    fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {}
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

Actions can be added to any `Entity` with the `SequentialActions` marker component.
Adding and modifying actions is done through the `actions(agent)`
extension method implemented for both `Commands` and `World`.
See the `ModifyActions` trait for available methods.

```rust
fn setup(mut commands: Commands) {
    // Spawn entity with the marker component
    let agent = commands.spawn(SequentialActions).id();
    commands
        .actions(agent)
        // Add a single action
        .add(action_a)
        // Add more actions with a tuple
        .add((action_b, action_c))
        // Add a collection of actions
        .add(actions![action_d, action_e, action_f])
        // Add an anonymous action with a closure
        .add(|_agent, world: &mut World| -> bool {
            // on_start
            world.send_event(AppExit::Success);
            true
        });
}
```

#### ⚠️ Warning

Since you are given a mutable `World`, you can in practice do _anything_.
Depending on what you do, the logic for advancing the action queue might not work properly.
There are a few things you should keep in mind:

* If you want to despawn an `agent` as an action, this should be done in `on_start`.
* The `execute` and `next` methods should not be used,
    as that will immediately advance the action queue while inside any of the trait methods.
    Instead, you should return `true` in `on_start`.
* When adding new actions, you should set the `start` property to `false`.
    Otherwise, you will effectively call `execute` which, again, should not be used.
    At worst, you will cause a **stack overflow** if the action adds itself.

    ```rust,no_run
        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            world
                .actions(agent)
                .start(false) // Do not start next action
                .add((action_a, action_b, action_c));

            // Immediately advance the action queue
            true
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
| 0.16 | 0.13                    |
| 0.15 | 0.12                    |
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