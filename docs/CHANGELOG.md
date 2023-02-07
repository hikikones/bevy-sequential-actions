# Changelog

## Version 0.7.0-dev

<!-- TODO -->

## Version 0.6.0

- [Update to Bevy 0.9][55]
- [Add parallel actions][45]
- [Add `SequentialActionsPlugin::get_systems` for scheduling the systems yourself][53]
- [Add `actions!` helper macro for creating a collection of boxed actions][47]
- [Add `Repeat::None`][50]
- [Add `ActionsBundle::new`][52]
- [Rename `ActionCommands::custom` method to `add`][48]
- [Remove `ActionMarker` component][49]

## Version 0.5.0
- [Replace `builder` constructs with `add_many` method][40]
- [Replace repeat bool with a `Repeat` enum][41]

## Version 0.4.0

- [Add `ActionBuilder` trait][28]
- [Add `skip` method for skipping the next action in the queue][30]
- [Add `ActionMarker` component to `ActionsBundle`][31]
- [Add an anonymous action using a closure][34]
- [Add deferred `World` mutation when modifying actions using `ActionCommands`][36]

## Version 0.3.0

- [Update to Bevy 0.8][26]
- [Rename `Action` trait methods, add `StopReason` enum and other stuff][25]

## Version 0.2.0

- [Relicense to dual MIT or Apache 2.0][13]
- [New simplified API for modifying actions][12]
- [Allow despawning an entity as its _last_ action][11]

## Version 0.1.0

First release! 🎉

[55]: https://github.com/hikikones/bevy-sequential-actions/pull/55
[53]: https://github.com/hikikones/bevy-sequential-actions/pull/53
[52]: https://github.com/hikikones/bevy-sequential-actions/pull/52
[50]: https://github.com/hikikones/bevy-sequential-actions/pull/50
[49]: https://github.com/hikikones/bevy-sequential-actions/pull/49
[48]: https://github.com/hikikones/bevy-sequential-actions/pull/48
[47]: https://github.com/hikikones/bevy-sequential-actions/pull/47
[45]: https://github.com/hikikones/bevy-sequential-actions/pull/45
[41]: https://github.com/hikikones/bevy-sequential-actions/pull/41
[40]: https://github.com/hikikones/bevy-sequential-actions/pull/40
[36]: https://github.com/hikikones/bevy-sequential-actions/pull/36
[34]: https://github.com/hikikones/bevy-sequential-actions/pull/34
[31]: https://github.com/hikikones/bevy-sequential-actions/pull/31
[30]: https://github.com/hikikones/bevy-sequential-actions/pull/30
[28]: https://github.com/hikikones/bevy-sequential-actions/pull/28
[26]: https://github.com/hikikones/bevy-sequential-actions/pull/26
[25]: https://github.com/hikikones/bevy-sequential-actions/pull/25
[13]: https://github.com/hikikones/bevy-sequential-actions/pull/13
[12]: https://github.com/hikikones/bevy-sequential-actions/pull/12
[11]: https://github.com/hikikones/bevy-sequential-actions/pull/11