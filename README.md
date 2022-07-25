# Bevy Sequential Actions

`bevy_sequential_actions` is a library for the [Bevy game engine](https://bevyengine.org/ "bevy game engine") that aims to execute a list of actions in a sequential manner. This generally means that one action runs at a time, and when it is done, the next action will start, and so on until the list is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

## Getting Started

An action is anything that implements the `Action` trait, and can be added to any `Entity` that contains the `ActionsBundle`. Each action must signal when they are finished, which is done by calling the `next` method from either `Commands` or `ActionCommands`.

```rust
use bevy::prelude::*;
use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_startup_system(setup)
        .add_system(wait)
        .run();
}

fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let entity = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single action with default config
    commands.action(entity).add(WaitAction(1.0));

    // Add multiple actions with custom config
    commands
        .action(entity)
        .config(AddConfig {
            // Add each action to the back of the queue
            order: AddOrder::Back,
            // Start action if nothing is currently running
            start: false,
            // Repeat the action
            repeat: false,
        })
        .add(WaitAction(2.0))
        .add(WaitAction(3.0));
}

struct WaitAction(f32);

impl Action for WaitAction {
    fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(entity).insert(Wait(self.0));
    }

    fn stop(&mut self, entity: Entity, world: &mut World) {
        world.entity_mut(entity).remove::<Wait>();
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
    for (entity, mut wait) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            // Action is finished, issue next.
            commands.action(entity).next();
        }
    }
}
```

## Examples

See the [examples](examples/) for more usage. Each example can be run with `cargo run --example <example>`.

| Example    | Description                                                                                                                                                                            |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `basic`    | Shows the basic usage of the library by adding a bunch of actions.                                                                                                                     |
| `stop`     | Shows how to stop a running action, and then add a new action to the front of the queue.                                                                                               |
| `repeat`   | Shows how to add actions that basically loop forever in the added order.                                                                                                               |
| `callback` | Shows an action with a callback.                                                                                                                                                       |
| `demo`     | A more comprehensive and practical example showcasing how this library can be used in a turn-based board game. Includes lots of custom actions that can be reused throughout the game. |

## Compatibility

| bevy | bevy_sequential_actions |
| ---- | ----------------------- |
| 0.7  | 0.1 — 0.2               |
