# Bevy Sequential Actions

A [Bevy](https://bevyengine.org) library that aims to execute a list of various actions in a sequential manner. This generally means that one action runs at a time, and when it is done, the next action will start and so on until the list is empty.

https://user-images.githubusercontent.com/19198785/167969191-48258eb3-8acb-4f38-a326-f34e055a1b40.mp4

## Getting Started

An action is anything that implements the `Action` trait, and can be added to any `Entity` that contains the `ActionsBundle`. An entity with actions is referred to as an `agent`.

```rust
fn setup(mut commands: Commands) {
    // Create entity with ActionsBundle
    let agent = commands.spawn_bundle(ActionsBundle::default()).id();
    
    // Add a single action with default config
    commands.actions(agent).add(wait_action);
    
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
| `pause`  | Shows how to pause and resume an action when pressing `space`.                         |
| `repeat` | Shows how to add actions that repeat `n` times and forever.                            |

## Compatibility

| bevy | bevy-sequential-actions |
| ---- | ----------------------- |
| 0.8  | 0.3 — 0.5               |
| 0.7  | 0.1 — 0.2               |
