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

    /// Starts the next [`action`](Action) in the queue by [`stopping`](Action::stop) the currently running action,
    /// and [`starting`](Action::start) the next action in the queue list.
    fn next(self) -> Self;

    /// [`Stops`](Action::stop) the currently running [`action`](Action) without removing it from the queue.
    fn stop(self) -> Self;

    /// [`Stops`](Action::stop) the currently running [`action`](Action), and clears any remaining.
    fn clear(self) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions __will not__ be added to the queue until [`submit`](Self::submit) is called.
    fn push(self, action: impl IntoAction) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submit the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
