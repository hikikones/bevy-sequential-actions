# Changelog

## Version 0.12.0

- [Update to Bevy 0.15][103]
- [Add `SequentialActions` marker for required components][106]
- [Add skip amount in `ModifyActions::skip`][105]
- [Add multiple actions with a tuple][104]
    - Removes `add_many` as `add` can now be used instead
- [Add cleanup of actions for despawned agents][101]
    - `on_stop`, `on_remove` and `on_drop` in the `Action` trait
    now take `Option<Entity>` for `agent` in order to respond to despawns
    - Replaces all unwraps with logging
    - Adds `on_remove_hook` and `on_remove_trigger` for both `CurrentAction` and `ActionQueue`

[106]: https://github.com/hikikones/bevy-sequential-actions/pull/106
[105]: https://github.com/hikikones/bevy-sequential-actions/pull/105
[104]: https://github.com/hikikones/bevy-sequential-actions/pull/104
[103]: https://github.com/hikikones/bevy-sequential-actions/pull/103
[101]: https://github.com/hikikones/bevy-sequential-actions/pull/101

## Version 0.11.0

- [Update to Bevy 0.14][97]

[97]: https://github.com/hikikones/bevy-sequential-actions/pull/97

## Version 0.10.0

- [Update to Bevy 0.13][95]
- [fix: Removed Unwrap from plugin agent check][94]

[95]: https://github.com/hikikones/bevy-sequential-actions/pull/95
[94]: https://github.com/hikikones/bevy-sequential-actions/pull/94

## Version 0.9.0

- [Update to Bevy 0.12][91]
- [Move `ActionHandler` methods to `SequentialActionsPlugin`][90]

[91]: https://github.com/hikikones/bevy-sequential-actions/pull/91
[90]: https://github.com/hikikones/bevy-sequential-actions/pull/90

## Version 0.8.0

- [Update to Bevy 0.11][85]
- [Add downcasting for boxed actions][84]
- [Rework actions to be both more composable and simpler][83]
    - Adds four new methods to the `Action` trait:
        - `is_finished` which determines if an action is finished or not.
        - `on_add` which is called when an action is added to the queue.
        - `on_remove` which is called when an action is removed from the queue.
        - `on_drop` which is the last method to be called and gives full ownership.
    - Changes `Action::on_start` to now return a `bool` for immediate action queue advancement.
    - Removes `ActionCommands` struct for modifying actions inside the action trait.
    - Removes `ActionFinished` component.
    - Removes `Repeat` configuration.
    - Removes parallel and linked actions.
    - Removes tuple closure for anonymous actions with both `on_start` and `on_stop`.
    - Renames `add_sequential` to `add_many`.
    - Exposes the `ActionQueue` and `CurrentAction` components used by agents.
    - Exposes the `ActionHandler` struct that contains the system and methods used by this library.

[85]: https://github.com/hikikones/bevy-sequential-actions/pull/85
[84]: https://github.com/hikikones/bevy-sequential-actions/pull/84
[83]: https://github.com/hikikones/bevy-sequential-actions/pull/83

## Version 0.7.0

- [Update to Bevy 0.10][73]
- [Add linked actions][63]
    - Replaces the `add_many` method with `add_sequence` and `add_parallel`
- [Add `execute` method to `ModifyActions` trait][68]
- [Replace `config` with `start`, `order` and `repeat` methods][64]
- [Replace `IntoBoxedAction` trait with `From<Box<dyn Action>>`][65]
- [Don't advance the action queue when canceling][67]
- [Use `Command` and `CommandQueue` in `ActionCommands`][71]

[73]: https://github.com/hikikones/bevy-sequential-actions/pull/73
[71]: https://github.com/hikikones/bevy-sequential-actions/pull/71
[68]: https://github.com/hikikones/bevy-sequential-actions/pull/68
[67]: https://github.com/hikikones/bevy-sequential-actions/pull/67
[65]: https://github.com/hikikones/bevy-sequential-actions/pull/65
[64]: https://github.com/hikikones/bevy-sequential-actions/pull/64
[63]: https://github.com/hikikones/bevy-sequential-actions/pull/63

## Version 0.6.0

- [Update to Bevy 0.9][55]
- [Add parallel actions][45]
    - Introduces the `SequentialActionsPlugin`
    - Replaces the `finish` method with the `ActionFinished` component
    - Replaces the `stop(reason)` method with `cancel` and `pause`
- [Add `SequentialActionsPlugin::get_systems` for scheduling the systems yourself][53]
- [Add `actions!` helper macro for creating a collection of boxed actions][47]
- [Add `Repeat::None`][50]
- [Add `ActionsBundle::new`][52]
- [Rename `ActionCommands::custom` method to `add`][48]
- [Remove `ActionMarker` component][49]

[55]: https://github.com/hikikones/bevy-sequential-actions/pull/55
[53]: https://github.com/hikikones/bevy-sequential-actions/pull/53
[52]: https://github.com/hikikones/bevy-sequential-actions/pull/52
[50]: https://github.com/hikikones/bevy-sequential-actions/pull/50
[49]: https://github.com/hikikones/bevy-sequential-actions/pull/49
[48]: https://github.com/hikikones/bevy-sequential-actions/pull/48
[47]: https://github.com/hikikones/bevy-sequential-actions/pull/47
[45]: https://github.com/hikikones/bevy-sequential-actions/pull/45

## Version 0.5.0
- [Replace `builder` constructs with `add_many` method][40]
- [Replace repeat bool with a `Repeat` enum][41]

[41]: https://github.com/hikikones/bevy-sequential-actions/pull/41
[40]: https://github.com/hikikones/bevy-sequential-actions/pull/40

## Version 0.4.0

- [Add `ActionBuilder` trait][28]
- [Add `skip` method for skipping the next action in the queue][30]
- [Add `ActionMarker` component to `ActionsBundle`][31]
- [Add an anonymous action using a closure][34]
- [Add deferred `World` mutation when modifying actions using `ActionCommands`][36]

[36]: https://github.com/hikikones/bevy-sequential-actions/pull/36
[34]: https://github.com/hikikones/bevy-sequential-actions/pull/34
[31]: https://github.com/hikikones/bevy-sequential-actions/pull/31
[30]: https://github.com/hikikones/bevy-sequential-actions/pull/30
[28]: https://github.com/hikikones/bevy-sequential-actions/pull/28

## Version 0.3.0

- [Update to Bevy 0.8][26]
- [Rename `Action` trait methods, add `StopReason` enum and other stuff][25]
    - Adds `StopReason` enum for `Action::on_stop` method
    - Removes the `Action::remove` trait method
    - Renames `Action::start` method to `Action::on_start`
    - Renames `Action::stop` method to `Action::on_stop`
    - Renames `action(entity)` method to `actions(entity)` for modifying actions

[26]: https://github.com/hikikones/bevy-sequential-actions/pull/26
[25]: https://github.com/hikikones/bevy-sequential-actions/pull/25

## Version 0.2.0

- [Relicense to dual MIT or Apache 2.0][13]
- [New simplified API for modifying actions][12]
    - Renames `Action::add` method to `Action::start`
- [Allow despawning an entity as its _last_ action][11]

[13]: https://github.com/hikikones/bevy-sequential-actions/pull/13
[12]: https://github.com/hikikones/bevy-sequential-actions/pull/12
[11]: https://github.com/hikikones/bevy-sequential-actions/pull/11

## Version 0.1.0

First release! 🎉
