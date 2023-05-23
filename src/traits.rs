use crate::*;

/// The trait that all actions must implement.
#[allow(unused_variables)]
pub trait Action: Send + Sync + 'static {
    /// Determines if an action is finished or not.
    /// Advances the action queue when returning `true`.
    ///
    /// By default, this method is called every frame in [`CoreSet::Last`](bevy_app::CoreSet::Last).
    fn is_finished(&self, agent: Entity, world: &World) -> Finished;

    /// The method that is called when an action is started.
    ///
    /// Returning `true` here marks the action as already finished,
    /// and will immediately advance the action queue.
    fn on_start(&mut self, agent: Entity, world: &mut World) -> Finished;

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason);

    /// The method that is called when an action is added to the queue.
    fn on_add(&mut self, agent: Entity, world: &mut World) {}

    /// The method that is called when an action is removed from the queue.
    fn on_remove(&mut self, agent: Entity, world: &mut World) {}

    /// The last method that is called for an action.
    /// Full ownership is given here, hence the name.
    fn on_drop(self: Box<Self>, agent: Entity, world: &mut World, reason: DropReason) {}
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
    OnStart: FnMut(Entity, &mut World) -> Finished + Send + Sync + 'static,
{
    fn is_finished(&self, _agent: Entity, _world: &World) -> Finished {
        Finished(true)
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> Finished {
        (self)(agent, world)
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
    /// Specify if the next [`action`](Action) in the queue should
    /// [`start`](Action::on_start) when new actions are added.
    /// The next action will only start if there is no current action.
    ///
    /// Default is `true`.
    fn start(&mut self, start: bool) -> &mut Self;

    /// Specify the queue order for actions to be added.
    ///
    /// Default is [`AddOrder::Back`].
    fn order(&mut self, order: AddOrder) -> &mut Self;

    /// Adds a single [`action`](Action) to the queue.
    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self;

    /// Adds a collection of actions to the queue.
    fn add_many(
        &mut self,
        actions: impl IntoIterator<
                Item = BoxedAction,
                IntoIter = impl DoubleEndedIterator<Item = BoxedAction>,
            > + Send
            + 'static,
    ) -> &mut Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue,
    /// but only if there is no current action.
    fn execute(&mut self) -> &mut Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn next(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current action as [`canceled`](StopReason::Canceled).
    ///
    /// To resume the action queue, call either [`execute`](Self::execute) or [`next`](Self::next).
    fn cancel(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current action as [`paused`](StopReason::Paused).
    ///
    /// To resume the action queue, call either [`execute`](Self::execute) or [`next`](Self::next).
    fn pause(&mut self) -> &mut Self;

    /// Skips the next [`action`](Action) in the queue.
    fn skip(&mut self) -> &mut Self;

    /// Clears the action queue.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn clear(&mut self) -> &mut Self;
}
