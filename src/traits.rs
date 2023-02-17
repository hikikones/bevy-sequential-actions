use crate::*;

/// The trait that all actions must implement.
pub trait Action: Send + Sync + 'static {
    /// The method that is called when an action is started.
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut ActionCommands);

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason);
}

impl<T> From<T> for BoxedAction
where
    T: Action,
{
    fn from(action: T) -> Self {
        Box::new(action)
    }
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

/// Proxy method for modifying actions. Returns a type that implements [`ModifyActions`].
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns a type for modifying actions for specified `agent`.
    fn actions(&'a mut self, agent: Entity) -> Self::Modifier;
}

/// Methods for modifying actions.
pub trait ModifyActions {
    /// Specify if the next [`action`](Action) in the queue should [`start`](Action::on_start) when added.
    /// It will only start if nothing is currently running.
    /// Default is `true`.
    fn start(&mut self, start: bool) -> &mut Self;

    /// Specify the queue order for actions to be added.
    /// Default is [`AddOrder::Back`].
    fn order(&mut self, order: AddOrder) -> &mut Self;

    /// Specify the repeat configuration for actions to be added.
    /// Default is [`Repeat::None`].
    fn repeat(&mut self, repeat: Repeat) -> &mut Self;

    /// Adds a single [`action`](Action) to the queue.
    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self;

    /// Adds a collection of actions to the queue that are executed sequentially, i.e. one by one.
    fn add_sequence(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self;

    /// Adds a collection of actions to the queue that are executed in parallel, i.e. all at once.
    fn add_parallel(
        &mut self,
        actions: impl Iterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self;

    /// Adds a collection of _linked_ actions to the queue that are executed sequentially.
    /// Linked actions have the property that if any of them are [`canceled`](ModifyActions::cancel),
    /// then the remaining actions in the collection are ignored.
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
