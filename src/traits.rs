use bevy_ecs::prelude::*;

use crate::*;

/// Conversion into an [`Action`].
pub trait IntoAction {
    /// Convert `self` into `Box<dyn Action>`.
    fn into_boxed(self) -> Box<dyn Action>;
}

impl<T> IntoAction for T
where
    T: Action + 'static,
{
    fn into_boxed(self) -> Box<dyn Action> {
        Box::new(self)
    }
}

impl IntoAction for Box<dyn Action> {
    fn into_boxed(self) -> Box<dyn Action> {
        self
    }
}

pub trait ActionsExt {
    fn config(self, config: AddConfig) -> Self;
    fn add(self, action: impl IntoAction) -> Self;
    fn stop(self) -> Self;
    fn next(self) -> Self;
    fn clear(self) -> Self;
    fn push(self, action: impl IntoAction) -> Self;
    fn reverse(self) -> Self;
    fn submit(self) -> Self;
}

/// Extension trait for `add_action` method on [`ActionCommands`] and [`Commands`].
pub trait AddActionExt {
    /// Add `action` to entity `actor` with the configuration `config`.
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig);
}

/// Extension trait for `stop_action` method on [`ActionCommands`] and [`Commands`].
pub trait StopActionExt {
    /// Stop the current action for entity `actor`. This is done by removing the currently running action,
    /// and pushing it to the **front** of the queue again.
    ///
    /// **Note**: when stopping an action, you need to manually resume the actions.
    /// This can be done by calling `next_action`, which will resume the same action that was stopped,
    /// or you could add a new action to the **front** of the queue beforehand.
    /// When adding a new action, either specify in [`AddConfig`] that the action should start,
    /// or manually call `next_action` afterwards, but not both, as that will trigger two
    /// consecutive `next_action` calls.
    ///
    /// # Example
    ///
    /// Stopping an [`Action`] and adding a new one with `start: true` in [`AddConfig`]:
    ///
    /// ```rust
    /// commands.stop_action(actor);
    /// commands.add_action(
    ///     actor,
    ///     MyAction,
    ///     AddConfig {
    ///         order: AddOrder::Front,
    ///         start: true,
    ///         repeat: false,
    ///     },
    /// );
    /// // No need to call next_action here
    /// ```
    ///
    /// Stopping an [`Action`] and manually calling `next_action`:
    ///
    /// ```rust
    /// commands.stop_action(actor);
    /// commands.add_action(
    ///     actor,
    ///     MyAction,
    ///     AddConfig {
    ///         order: AddOrder::Front,
    ///         start: false,
    ///         repeat: false,
    ///     },
    /// );
    /// // Must call next_action here
    /// commands.next_action(actor);
    /// ```
    fn stop_action(&mut self, actor: Entity);
}

/// Extension trait for `next_action` method on [`ActionCommands`] and [`Commands`].
pub trait NextActionExt {
    /// Start next action for entity `actor`. This is done by removing the currently running action,
    /// and retrieving the next action in the queue list.
    fn next_action(&mut self, actor: Entity);
}

/// Extension trait for `clear_actions` method on [`ActionCommands`] and [`Commands`].
pub trait ClearActionsExt {
    /// Remove the currently running action for entity `actor`, and clear any remaining.
    fn clear_actions(&mut self, actor: Entity);
}
