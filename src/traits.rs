use crate::*;

/// The trait that all actions must implement.
pub trait Action: Send + Sync + 'static {
    /// The method that is called when an action is started.
    fn on_start(&mut self, state: &mut WorldState, commands: &mut ActionCommands);

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, state: &mut WorldState, reason: StopReason);
}

impl<Start> Action for Start
where
    Start: FnMut(&mut WorldState, &mut ActionCommands) + Send + Sync + 'static,
{
    fn on_start(&mut self, state: &mut WorldState, commands: &mut ActionCommands) {
        (self)(state, commands);
    }

    fn on_stop(&mut self, _state: &mut WorldState, _reason: StopReason) {}
}

impl<Start, Stop> Action for (Start, Stop)
where
    Start: FnMut(&mut WorldState, &mut ActionCommands) + Send + Sync + 'static,
    Stop: FnMut(&mut WorldState, StopReason) + Send + Sync + 'static,
{
    fn on_start(&mut self, state: &mut WorldState, commands: &mut ActionCommands) {
        (self.0)(state, commands);
    }

    fn on_stop(&mut self, state: &mut WorldState, reason: StopReason) {
        (self.1)(state, reason);
    }
}

/// Conversion into a [`BoxedAction`].
pub trait IntoBoxedAction: Send + Sync + 'static {
    /// Convert `self` into [`BoxedAction`].
    fn into_boxed(self) -> BoxedAction;
}

impl<T> IntoBoxedAction for T
where
    T: Action,
{
    fn into_boxed(self) -> BoxedAction {
        Box::new(self)
    }
}

impl IntoBoxedAction for BoxedAction {
    fn into_boxed(self) -> BoxedAction {
        self
    }
}

/// Trait alias for a collection of actions.
pub trait BoxedActionIter: DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static {}

impl<T> BoxedActionIter for T where
    T: DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static
{
}

/// Proxy method for modifying actions. Returns a type that implements [`ModifyActions`].
///
/// # Warning
///
/// Do not modify actions using [`World`] inside the implementation of an [`Action`].
/// Actions need to be properly queued, which is what [`ActionCommands`] does.
/// If you need to use [`World`] for modifying actions, use [`EntityActions::custom`].
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Bad
///         world.actions(agent).next();
///
///         // Good
///         commands.actions(agent).next();
///
///         // Also good
///         commands.custom(move |w: &mut World| {
///             w.actions(agent).next();
///         });
///
///         // Also good if you want to mark it as finished
///         world.get_mut::<ActionFinished>(agent).unwrap().confirm();
///     }
///
///     fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {}
/// }
///```
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns [`Self::Modifier`] for specified [`agent`](Entity).
    fn actions(&'a mut self, agent: Entity) -> Self::Modifier;
}

/// Methods for modifying actions.
pub trait ModifyActions {
    /// Sets the current [`config`](AddConfig) for actions to be added.
    fn config(self, config: AddConfig) -> Self;

    /// Adds an [`action`](Action) to the queue with the current [`config`](AddConfig).
    fn add(self, action: impl IntoBoxedAction) -> Self;

    /// Adds a collection of [`actions`](Action) to the queue with the current [`config`](AddConfig).
    fn add_many(self, mode: ExecutionMode, actions: impl BoxedActionIter) -> Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn next(self) -> Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`canceled`](StopReason::Canceled).
    fn cancel(self) -> Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`paused`](StopReason::Paused).
    fn pause(self) -> Self;

    /// Skips the next [`action`](Action) in the queue.
    fn skip(self) -> Self;

    /// Clears the actions queue.
    /// Current [`action`](Action) is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn clear(self) -> Self;
}
