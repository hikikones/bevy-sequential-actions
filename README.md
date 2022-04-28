# Bevy Sequential Actions

`bevy_sequential_actions` is a library for the [Bevy game engine](https://bevyengine.org/ "bevy game engine") that aims to execute a list of actions in a sequential manner. This generally means that one action runs at a time, and when it is done, the next action will start, and so on until the list is empty.

## Getting Started

An action is anything that implements the `Action` trait, and can be added to any `Entity` that contains the `ActionsBundle`. Each action must signal when they are finished, which is done by calling `next_action` on either `Commands` or `ActionCommands`.

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
    let id = commands.spawn_bundle(ActionsBundle::default()).id();

    // Add a single action with default config
    commands.add_action(id, WaitAction(1.0), AddConfig::default());

    // Add multiple actions with custom config
    commands
        .action_builder(
            id,
            AddConfig {
                // Add each action to the back of the queue
                order: AddOrder::Back,
                // Start action if nothing is currently running
                start: false,
                // Repeat the action         
                repeat: false,
            },
        )
        .push(WaitAction(2.0))
        .push(WaitAction(3.0))
        .submit();
}

struct WaitAction(f32);

impl Action for WaitAction {
    fn add(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        world.entity_mut(actor).insert(Wait(self.0));
    }

    fn remove(&mut self, actor: Entity, world: &mut World) {
        world.entity_mut(actor).remove::<Wait>();
    }

    fn stop(&mut self, actor: Entity, world: &mut World) {
        self.remove(actor, world);
    }
}

#[derive(Component)]
struct Wait(f32);

fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
    for (actor, mut wait) in wait_q.iter_mut() {
        wait.0 -= time.delta_seconds();
        if wait.0 <= 0.0 {
            // Action is finished, issue next.
            commands.next_action(actor);
        }
    }
}
```