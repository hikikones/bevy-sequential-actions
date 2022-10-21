# Bevy Sequential Actions

A [Bevy](https://bevyengine.org) library that aims to execute a list of various actions in a sequential manner.
This generally means that one action runs at a time, and when it is done,
the next action will start and so on until the list is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

## Getting Started

### Plugin

In order for everything to work, the `SequentialActionsPlugin` must be added to your `App`.

```rust
use bevy::prelude::*;
use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SequentialActionsPlugin)
        .run();
}
```

### Modifying Actions

An action is anything that implements the `Action` trait, and can be added to any `Entity` that contains the `ActionsBundle`. An entity with actions is referred to as an `agent`.

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
            repeat: Repeat::Amount(0),
        })
        .add(action_b)
        .add(action_c);

    // Add a collection of actions that run in parallel.
    // This means that all actions will start at the same time,
    // and stop when all are finished within the same frame.
    commands
        .actions(agent)
        .add_many(
            ExecutionMode::Parallel,
            [
                action_a.into_boxed(),
                action_b.into_boxed(),
                action_c.into_boxed(),
            ].into_iter()
        );
}
```

### Implementing an Action

The `Action` trait contains two methods:

* The `on_start` method which is called when an action is started.
* The `on_stop` method which is called when an action is stopped.

Each action is given a unique entity that we will refer to as the `status` entity.
This entity is spawned before an action starts, and despawned after it stops.
It contains two components:

* The `ActionFinished` component which must be used in order to declare that an action is finished.
* The `ActionAgent` component which is optionally used for getting the entity ID for the `agent`.

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

        // Run the wait system on the given status entity
        world.entity_mut(id.status()).insert(Wait(duration));
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        // Store current duration when paused
        if let StopReason::Paused = reason {
            self.current = Some(world.get::<Wait>(id.status()).unwrap().0);
        }
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait_system(mut wait_q: Query<(&mut Wait, &mut ActionFinished)>, time: Res<Time>) {
    for (mut wait, mut finished) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();

        // Set the finished status for the action
        finished.set(wait.0 <= 0.0);
    }
}
```

### Warning

One thing to keep in mind is that you should not modify actions using `World` inside the `Action` trait.
In order to pass a mutable `World`, the current action is temporarily removed from the `agent`
before either of the trait methods are called, and put back again after.
This is why `ActionCommands` was created, so you can safely modify actions inside the `Action` trait
in a deferred way.

```rust
pub struct SetStateAction<T: StateData>(T);

impl<T: StateData> Action for SetStateAction<T> {
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        world
            .resource_mut::<State<T>>()
            .set(self.0.clone())
            .unwrap();

        // Bad
        world.actions(id.agent()).next();
        
        // Good
        commands.actions(id.agent()).next();
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
```

Another thing to keep in mind is that there are two ways to advance the actions queue:

* Using the `ActionFinished` component. A system at the end of the frame will advance the queue if all active actions for an `agent` are finished. This is the recommended approach as it composes well with other actions running in parallel.
* Calling the `next` method. This simply advances the queue at the end of the current stage it was called in. Useful for small one-at-a-time actions where you want to advance the queue more quickly (instead of waiting a whole frame).

Note that the first approach works with actions in parallel, while the second approach does not.

## Examples

See the [examples](examples/) for more usage, specifically the [shared actions](examples/shared/src/actions/).
Each example can be run with `cargo run --example <example>`.
Consider running with `--release` as debug builds can be quite slow.

| Example  | Description                                                                            |
| -------- | -------------------------------------------------------------------------------------- |
| `basic`  | Shows the basic usage of the library by adding some actions and then quitting the app. |
| `pause`  | Shows how to pause and resume an action when pressing `space`.                         |
| `repeat` | Shows how to add actions that repeat `n` times and forever.                            |

## Compatibility

| bevy | bevy-sequential-actions |
| ---- | ----------------------- |
| 0.8  | 0.3 — 0.5               |
| 0.7  | 0.1 — 0.2               |
