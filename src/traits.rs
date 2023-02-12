use crate::*;

/// The trait that all actions must implement.
pub trait Action: Send + Sync + 'static {
    /// The method that is called when an action is started.
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands);

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason);
}

impl<Start> Action for Start
where
    Start: FnMut(Entity, &mut World, &mut ActionCommands) + Send + Sync + 'static,
{
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        (self)(agent, world, commands);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

impl<Start, Stop> Action for (Start, Stop)
where
    Start: FnMut(Entity, &mut World, &mut ActionCommands) + Send + Sync + 'static,
    Stop: FnMut(Entity, &mut World, StopReason) + Send + Sync + 'static,
{
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands) {
        (self.0)(agent, world, commands);
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        (self.1)(agent, world, reason);
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
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns a type for modifying actions for specified `agent`.
    fn actions(&'a mut self, agent: Entity) -> Self::Modifier;
}

/// Methods for modifying actions.
pub trait ModifyActions {
    /// Sets the current [`config`](AddConfig) for actions to be added.
    fn config(&mut self, config: AddConfig) -> &mut Self;

    fn add(&mut self, action: impl IntoBoxedAction) -> &mut Self;

    fn add_sequence(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self;

    fn add_parallel(
        &mut self,
        actions: impl Iterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self;

    fn add_linked(
        &mut self,
        f: impl FnOnce(&mut LinkedActionsBuilder) + Send + Sync + 'static,
    ) -> &mut Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn next(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`canceled`](StopReason::Canceled).
    fn cancel(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`paused`](StopReason::Paused).
    fn pause(&mut self) -> &mut Self;

    /// Skips the next [`action`](Action) in the queue.
    fn skip(&mut self) -> &mut Self;

    /// Clears the action queue.
    /// Current [`action`](Action) is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn clear(&mut self) -> &mut Self;
}
