use crate::*;

/// The trait that all actions must implement.
///
/// # Example
///
/// An empty action that does nothing.
/// All actions must declare when they are done.
/// This is done by calling [`next`](ModifyActionsExt::next) from either [`ActionCommands`] or [`Commands`].
///
/// ```rust
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Action is finished, issue next.
///         commands.action(entity).next();
///     }
///
///     fn remove(&mut self, entity: Entity, world: &mut World) {}
///     fn stop(&mut self, entity: Entity, world: &mut World) {}
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an [`action`](Action) is started.
    fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands);
    /// The method that is called when an [`action`](Action) is removed.
    fn remove(&mut self, entity: Entity, world: &mut World);
    /// The method that is called when an [`action`](Action) is stopped.
    fn stop(&mut self, entity: Entity, world: &mut World);
}

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

/// Extension methods for modifying actions.
pub trait ModifyActionsExt {
    /// Sets the current [`config`](AddConfig) for actions to be added.
    fn config(self, config: AddConfig) -> Self;

    /// Adds an [`action`](Action) to the queue with the current [`config`](AddConfig).
    fn add(self, action: impl IntoAction) -> Self;

    /// Starts the next [`action`](Action) in the queue. This is done by [`removing`](Action::remove) the currently running action,
    /// and retrieving the next action in the queue list.
    fn next(self) -> Self;

    /// Stops the current [`action`](Action). This is done by [`removing`](Action::remove) the currently running action,
    /// and adding it to the **front** of the queue again.
    ///
    /// **Note:** when stopping an action, you need to manually resume again.
    /// This can be done by calling [`next`](Self::next), which will resume the same action that was stopped,
    /// or you could add a new action to the **front** of the queue beforehand.
    /// When adding a new action, either specify in [`config`](AddConfig) that the action should [`start`](AddConfig::start),
    /// or manually call [`next`](Self::next) afterwards, __but not both__, as that will trigger two
    /// consecutive [`next`](Self::next) calls.
    fn stop(self) -> Self;

    /// [`Removes`](Action::remove) the currently running action, and clears any remaining.
    fn clear(self) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions __will not__ be added to the queue until [`submit`](Self::submit) is called.
    fn push(self, action: impl IntoAction) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submit the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
