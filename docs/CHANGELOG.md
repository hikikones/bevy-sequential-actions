# Changelog

## Version 0.6.0

- Update to Bevy 0.9. (#55)
- Add a collection of actions that run in parallel with the `add_many` method. The collection is treated as "one action", meaning that they all start and stop at the same time, and the action queue will only advance when all actions are finished within the same frame. For this to work, this library now requires that you add the `SequentialActionsPlugin` to your app, as a set of systems at the end of the frame now needs to check for all finished actions. The `finish` method has been removed, as declaring an action as finished is now done with the `ActionFinished` component on an `agent`. The explicit `stop(reason)` method has also been removed. (#45)
- Add `SequentialActionsPlugin::get_systems` if you want to schedule the systems yourself. (#53)
- Add `actions!` helper macro for creating a collection of boxed actions. (#47)
- Add `Repeat::None` variant. (#50)
- Add `ActionsBundle::new` method. (#52)
- Rename `ActionCommands::custom` method to `add`. (#48)
- Remove `ActionMarker` component. (#49)

## Version 0.5.0

- Replace `builder` constructs for adding a list of actions with a simple `add_many` method that takes an iterator of boxed actions. Reversing the list before adding to the front is no longer needed. (#40)
- Replace the repeat bool in `AddConfig` with a `Repeat` enum. You can now specify how many times an action should be repeated before it is permanently removed from the queue. A value of zero means that it will only run once. (#41)

## Version 0.4.0

- Building a list of actions is now done through the `builder` method when modifying actions. (#28)
- Add `skip` method for skipping the next action. (#30)
- Add `ActionMarker` component to `ActionsBundle`. (#31)
- Add an anonymous action using a closure. (#34)
- Add `custom` method to `ActionCommands` for deferred `World` mutation after `Action::on_start` has been called. Used for modifying actions using `World` inside the `Action` trait. (#36)

## Version 0.3.0

- Update to Bevy 0.8.
- Remove the `Action::remove` trait method.
- Rename `Action::start` method to `Action::on_start`.
- Rename `Action::stop` method to `Action::on_stop`.
- Add `StopReason` enum and use it as parameter in `Action::on_stop` method.
- Rename `action(entity)` method to `actions(entity)` for modifying actions.

## Version 0.2.0

- New simplified API for modifying actions.
- Allow despawning an entity as its _last_ action.
- Rename `Action::add` to `Action::start`.

## Version 0.1.0

First release! 🎉
