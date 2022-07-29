# Bevy Sequential Actions

`bevy-sequential-actions` is a library for the [Bevy](https://bevyengine.org) game engine that aims to execute a list of actions in a sequential manner. This generally means that one action runs at a time, and when it is done, the next action will start, and so on until the list is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

## Getting Started

An action is anything that implements the `Action` trait, and can be added to any `Entity` that contains the `ActionsBundle`.

```rust
fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let entity = commands.spawn_bundle(ActionsBundle::default()).id();
    
    // Add a single action with default config
    commands.actions(entity).add(wait_action);
    
    // Add multiple actions with custom config
    commands
        .actions(entity)
        .config(AddConfig {
            // Add each action to the back of the queue
            order: AddOrder::Back,
            // Start the next in the queue if nothing is currently running
            start: true,
            // Repeat the action by adding it back to the queue when it is removed
            repeat: false,
        })
        .add(move_action)
        .add(quit_action);
}
```

## Examples

See the [examples](examples/) for more usage. Each example can be run with `cargo run --example <example>`.
Consider running with `--release` as debug builds can be quite slow.

| Example  | Description                                                                            |
| -------- | -------------------------------------------------------------------------------------- |
| `basic`  | Shows the basic usage of the library by adding some actions and then quitting the app. |
| `stop`   | Shows how to pause and resume an action when pressing `space`.                         |
| `repeat` | Shows how to add actions that basically loop forever in the added order.               |

## Compatibility

| bevy | bevy-sequential-actions |
| ---- | ----------------------- |
| 0.7  | 0.1 — 0.2               |
