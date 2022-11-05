# Bevy Sequential Actions

A [Bevy](https://bevyengine.org) library that aims to execute a queue of various actions in a sequential manner.
This generally means that one action runs at a time, and when it is done,
the next action will start and so on until the queue is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

## Getting Started

#### Plugin

In order for everything to work, the `SequentialActionsPlugin` must be added to your `App`.

```rust
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

An action is anything that implements the `Action` trait, and can be added to any `Entity` that contains the `ActionsBundle`.
An entity with actions is referred to as an `agent`.

```rust
fn setup(mut commands: Commands) {
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

The `Action` trait contains two methods:

* The `on_start` method which is called when an action is started.
* The `on_stop` method which is called when an action is stopped.

Every action is responsible for advancing the queue.
There are two ways of doing this:

* Using the `ActionFinished` component on an `agent`.
  By default, a system at the end of the frame will advance the queue if all active actions are finished.
  This is the typical approach as it composes well with other actions running in parallel.
* Calling the `next` method on an `agent`.
  This simply advances the queue at the end of the current stage it was called in.
  Useful for short one-at-a-time actions.

A simple wait action follows.

```rust
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

One thing to keep in mind is that you should not modify actions using `World` inside the `Action` trait.
We cannot borrow a mutable action from an `agent` while also passing a mutable world to it.
Since an action is detached from an `agent` when the trait methods are called,
the logic for advancing the action queue will not work properly.

This is why `ActionCommands` was created, so you can modify actions inside the `Action` trait in a deferred way.

```rust
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

        // Also good. By default, the action queue will advance at the end of the frame.
        world.get_mut::<ActionFinished>(agent).unwrap().confirm_and_persist();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
```

## Examples

See the [examples](examples/) for more usage, specifically the [shared actions](examples/shared/src/actions/).
Each example can be run with `cargo run --example <example>`.
Consider running with `--release` as debug builds can be quite slow.

| Example    | Description                                                                            |
| ---------- | -------------------------------------------------------------------------------------- |
| `basic`    | Shows the basic usage of the library by adding some actions and then quitting the app. |
| `pause`    | Shows how to pause and resume an action when pressing `space`.                         |
| `repeat`   | Shows how to add actions that repeat `n` times and forever.                            |
| `parallel` | Shows how to add a collection of actions that run in parallel.                         |

## Compatibility

| bevy | bevy-sequential-actions |
| ---- | ----------------------- |
| 0.8  | 0.3 — 0.5               |
| 0.7  | 0.1 — 0.2               |
