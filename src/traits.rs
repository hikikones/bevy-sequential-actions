use super::*;

/// The trait that all actions must implement.
#[allow(unused_variables)]
pub trait Action: downcast_rs::Downcast + Send + Sync + 'static {
    /// Determines if an action is finished or not.
    /// Advances the action queue when returning `true`.
    ///
    /// By default, this method is called every frame in the [`Last`] schedule.
    fn is_finished(&self, agent: Entity, world: &World) -> bool;

    /// The method that is called when an action is started.
    ///
    /// Returning `true` here marks the action as already finished,
    /// and will immediately advance the action queue.
    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool;

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason);

    /// The method that is called when an action is added to the queue.
    fn on_add(&mut self, agent: Entity, world: &mut World) {}

    /// The method that is called when an action is removed from the queue.
    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {}

    /// The last method that is called for an action.
    /// Full ownership is given here, hence the name.
    fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {}

    /// Returns the type name of an action.
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}

downcast_rs::impl_downcast!(Action);

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
    OnStart: FnMut(Entity, &mut World) -> bool + Send + Sync + 'static,
{
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
        (self)(agent, world)
    }

    fn on_stop(&mut self, _agent: Option<Entity>, _world: &mut World, _reason: StopReason) {}
}

impl std::fmt::Debug for BoxedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

/// Methods for modifying actions.
pub trait ModifyActionsExt {
    /// Adds a single [`action`](Action) to the queue with a specified [`config`](AddConfig).
    fn add_action_with_config(&mut self, config: AddConfig, action: impl Action) -> &mut Self;

    /// Adds a collection of actions to the queue with a specified [`config`](AddConfig).
    fn add_actions_with_config<I>(&mut self, config: AddConfig, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator;

    /// Adds a single [`action`](Action) to the queue with a default [`config`](AddConfig).
    fn add_action(&mut self, action: impl Action) -> &mut Self {
        Self::add_action_with_config(self, AddConfig::default(), action)
    }

    /// Adds a collection of actions to the queue with a default [`config`](AddConfig).
    fn add_actions<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator,
    {
        Self::add_actions_with_config(self, AddConfig::default(), actions)
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue,
    /// but only if there is no current action.
    fn execute_actions(&mut self) -> &mut Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn next_action(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current action as [`canceled`](StopReason::Canceled).
    ///
    /// To resume the action queue,
    /// call either [`execute_actions`](`Self::execute_actions`) or [`next_action`](`Self::next_action`).
    fn cancel_action(&mut self) -> &mut Self;

    /// [`Stops`](Action::on_stop) the current action as [`paused`](StopReason::Paused).
    ///
    /// To resume the action queue,
    /// call either [`execute_actions`](`Self::execute_actions`) or [`next_action`](`Self::next_action`).
    fn pause_action(&mut self) -> &mut Self;

    /// Skips the next [`action`](Action) in the queue.
    fn skip_next_action(&mut self) -> &mut Self;

    /// Clears the action queue.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn clear_actions(&mut self) -> &mut Self;
}

#[deprecated(
    since = "0.12.0",
    note = "Replaced by ModifyActionsExt trait implemented for EntityCommands and EntityWorldMut."
)]
/// Proxy method for modifying actions.
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns a type for modifying actions for specified `agent`.
    fn actions(&'a mut self, agent: Entity) -> Self::Modifier;
}

#[deprecated(
    since = "0.12.0",
    note = "Replaced by ModifyActionsExt trait implemented for EntityCommands and EntityWorldMut."
)]
/// Methods for modifying actions.
pub trait ModifyActions {
    /// Sets the current [`config`](AddConfig) for actions to be added.
    fn config(&mut self, config: AddConfig) -> &mut Self;

    /// Sets the [`start`](AddConfig::start) field in the current [`config`](AddConfig).
    ///
    /// Default is `true`.
    fn start(&mut self, start: bool) -> &mut Self;

    /// Sets the [`order`](AddConfig::order) field in the current [`config`](AddConfig).
    ///
    /// Default is [`AddOrder::Back`].
    fn order(&mut self, order: AddOrder) -> &mut Self;

    /// Adds a single [`action`](Action) to the queue.
    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self;

    /// Adds a collection of actions to the queue.
    fn add_many<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator;

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
