use crate::*;

/// The trait that all actions must implement.
///
/// # Example
///
/// An empty action that does nothing.
/// All actions must declare when they are done.
/// This is done by calling [`next`](ModifyActions::next) from either [`ActionCommands`] or [`Commands`].
///
/// ```rust
/// struct EmptyAction;
/// impl Action for EmptyAction {
///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Action is finished, issue next.
///         commands.action(entity).next();
///     }
///
///     fn stop(&mut self, entity: Entity, world: &mut World) {}
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an action is started.
    fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands);
    /// The method that is called when an action is stopped.
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

/// Proxy method for modifying actions. Returns a type that implements [`ModifyActions`].
///
/// # Warning
///
/// Do not modify actions using [`World`] inside the implementation of an [`Action`].
/// Actions need to be properly queued, which is what [`ActionCommands`] does.
///
/// ```rust
/// struct EmptyAction;
/// impl Action for EmptyAction {
///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Bad
///         world.action(entity).next();
///
///         // Good
///         commands.action(entity).next();
///     }
///
///     fn stop(&mut self, entity: Entity, world: &mut World) {}
/// }
///```
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns [`Self::Modifier`] for specified [`Entity`].
    fn action(&'a mut self, entity: Entity) -> Self::Modifier;
}

/// Methods for modifying actions.
pub trait ModifyActions {
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

    /// Submits the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
