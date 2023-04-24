use crate::*;

/// The trait that all actions must implement.
#[allow(unused_variables)]
pub trait Action: Send + Sync + 'static {
    /// Advances the action queue when returning `true`.
    ///
    /// By default, this method is called every frame in [`CoreSet::Last`](bevy_app::CoreSet::Last).
    fn is_finished(&self, agent: Entity, world: &World) -> bool;

    /// Called when an action is started.
    fn on_start(&mut self, agent: Entity, world: &mut World);

    /// Called when an action is stopped.
    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason);

    /// Called when an action is added to the queue.
    fn on_add(&mut self, agent: Entity, world: &mut World) {}

    /// Called when an action is removed from the queue.
    fn on_remove(self: Box<Self>, agent: Entity, world: &mut World) {}
}

impl<T> From<T> for BoxedAction
where
    T: Action,
{
    fn from(action: T) -> Self {
        Box::new(action)
    }
}

impl<OnStart> Action for OnStart
where
    OnStart: FnMut(Entity, &mut World) + Send + Sync + 'static,
{
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        (self)(agent, world);
    }

    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

/// Proxy method for modifying actions.
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns a type for modifying actions for specified `agent`.
    fn actions(&'a mut self, agent: Entity) -> Self::Modifier;
}

/// Methods for modifying actions.
pub trait ModifyActions {
    /// Specify if the next [`action`](Action) in the queue should [`start`](Action::on_start)
    /// when new actions are added. The next action will only start if nothing is currently running.
    /// Default is `true`.
    fn start(&mut self, start: bool) -> &mut Self;

    /// Specify the queue order for actions to be added.
    /// Default is [`AddOrder::Back`].
    fn order(&mut self, order: AddOrder) -> &mut Self;

    /// Adds a single [`action`](Action) to the queue.
    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self;

    /// Adds a collection of actions to the queue that are executed sequentially, i.e. one by one.
    fn add_many(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue,
    /// but only if there is no action currently running.
    fn execute(&mut self) -> &mut Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn next(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`canceled`](StopReason::Canceled).
    /// To resume the action queue, call either [`execute`](Self::execute) or [`next`](Self::next).
    fn cancel(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`paused`](StopReason::Paused).
    /// To resume the action queue, call either [`execute`](Self::execute) or [`next`](Self::next).
    fn pause(&mut self) -> &mut Self;

    /// Skips the next [`action`](Action) in the queue.
    fn skip(&mut self) -> &mut Self;

    /// Clears the action queue.
    /// Current [`action`](Action) is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn clear(&mut self) -> &mut Self;
}

/// Trait alias for marker components for agents.
pub trait AgentMarker: Default + Component {}

impl<T> AgentMarker for T where T: Default + Component {}
