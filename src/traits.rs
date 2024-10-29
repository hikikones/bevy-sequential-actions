use super::*;

/// The trait that all actions must implement.
///
/// It contains various methods that together defines the _lifecycle_ of an action.
/// From this, you can create any action that can last as long as you like,
/// and do as much as you like.
///
/// In general, the act of _starting_ the action should happen inside [`on_start`](`Self::on_start`).
/// This is especially true for despawning an `agent`.
/// The other methods should be used for initializing and cleaning up.
/// See below for more information.
///
/// ## ⚠️ Warning
///
/// Since you are given a mutable [`World`], you can in practice do _anything_.
/// Depending on what you do, the logic for advancing the action queue might not work properly.
///
/// There are a few things you should keep in mind:
///
/// * If you want to despawn an `agent` as an action, this should be done in [`on_start`](`Self::on_start`).
/// * The [`execute`](`ModifyActions::execute`) and [`next`](`ModifyActions::next`) methods should not be used,
///     as that will immediately advance the action queue while inside any of the trait methods.
///     Instead, you should return `true` in [`on_start`](`Self::on_start`).
/// * When adding new actions, you should set the [`start`](`ModifyActions::start`) property to `false`.
///     Otherwise, you will effectively call [`execute`](`ModifyActions::execute`) which, again, should not be used.
///     At worst, you will cause a **stack overflow** if the action adds itself.
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_sequential_actions::*;
/// # struct EmptyAction;
/// # impl Action for EmptyAction {
/// #   fn is_finished(&self, _a: Entity, _w: &World) -> bool { true }
/// #   fn on_start(&mut self, _a: Entity, _w: &mut World) -> bool { true }
/// #   fn on_stop(&mut self, _a: Option<Entity>, _w: &mut World, _r: StopReason) {}
/// # }
/// # struct TestAction;
/// # impl Action for TestAction {
/// #   fn is_finished(&self, _a: Entity, _w: &World) -> bool { true }
///     fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
/// #       let action_a = EmptyAction;
/// #       let action_b = EmptyAction;
/// #       let action_c = EmptyAction;
///         world
///             .actions(agent)
///             .start(false) // Do not start next action
///             .add((action_a, action_b, action_c));
///
///         // Immediately advance the action queue
///         true
///     }
/// #   fn on_stop(&mut self, _a: Option<Entity>, _w: &mut World, _r: StopReason) {}
/// # }
/// ```
#[allow(unused_variables)]
pub trait Action: downcast_rs::Downcast + Send + Sync + 'static {
    /// Determines if an action is finished or not.
    /// Advances the action queue when returning `true`.
    ///
    /// By default, this method is called every frame in the [`Last`] schedule.
    fn is_finished(&self, agent: Entity, world: &World) -> bool;

    /// The method that is called when an action is started.
    ///
    /// Typically here you would insert components to `agent` or a new entity
    /// to make systems run. If you wish to despawn `agent` as an action,
    /// **this is where you should do it**.
    ///
    /// Returning `true` here marks the action as already finished,
    /// and will immediately advance the action queue.
    fn on_start(&mut self, agent: Entity, world: &mut World) -> bool;

    /// The method that is called when an action is stopped.
    ///
    /// Typically here you would clean up any stuff from [`on_start`](`Self::on_start`),
    /// depending on the [`reason`](`StopReason`).
    ///
    /// When paused, the action will be put back into the action queue again to the front.
    /// This means that it will start again when the next action is executed.
    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, reason: StopReason);

    /// The method that is called when an action is added to the queue.
    ///
    /// You can think of this as the _constructor_, as it will only be called once.
    fn on_add(&mut self, agent: Entity, world: &mut World) {}

    /// The method that is called when an action is removed from the queue.
    ///
    /// You can think of this as the _destructor_, as it will only be called once.
    fn on_remove(&mut self, agent: Option<Entity>, world: &mut World) {}

    /// The last method that is called with full ownership.
    ///
    /// This is useful for actions that might want to alter the behavior of the action queue.
    /// For example, a `RepeatAction` could keep readding itself to the action queue based on some counter.
    fn on_drop(self: Box<Self>, agent: Option<Entity>, world: &mut World, reason: DropReason) {}

    /// Returns the type name of an action.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

downcast_rs::impl_downcast!(Action);

impl std::fmt::Debug for BoxedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name())
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

/// Proxy method for modifying actions.
pub trait ActionsProxy {
    /// Returns a type for modifying actions for specified `agent`.
    fn actions(&mut self, agent: Entity) -> impl ModifyActions;
}

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

    /// Adds one or more actions to the queue.
    fn add(&mut self, actions: impl IntoBoxedActions) -> &mut Self;

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

/// Conversion of an [Action] to a [BoxedAction].
pub trait IntoBoxedAction {
    /// Converts `self` into [BoxedAction].
    fn into_boxed_action(self) -> BoxedAction;
}

impl<T> IntoBoxedAction for T
where
    T: Action,
{
    fn into_boxed_action(self) -> BoxedAction {
        Box::new(self)
    }
}

impl IntoBoxedAction for BoxedAction {
    fn into_boxed_action(self) -> BoxedAction {
        self
    }
}

/// Conversion of actions to a collection of boxed actions.
pub trait IntoBoxedActions {
    /// Converts `self` into collection of boxed actions.
    fn into_boxed_actions(
        self,
    ) -> impl DoubleEndedIterator<Item = BoxedAction> + ExactSizeIterator + Send + Debug + 'static;
}

impl<T: Action> IntoBoxedActions for T {
    fn into_boxed_actions(
        self,
    ) -> impl DoubleEndedIterator<Item = BoxedAction> + ExactSizeIterator + Send + Debug + 'static
    {
        [self.into_boxed_action()].into_iter()
    }
}

macro_rules! impl_action_tuple {
    ($($T:ident),+) => {
        impl<$($T:Action),+> IntoBoxedActions for ($($T,)+) {
            fn into_boxed_actions(
                self,
            ) -> impl DoubleEndedIterator<Item = BoxedAction> + ExactSizeIterator + Send + Debug + 'static
            {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                [$( $T.into_boxed_action() ),+].into_iter()
            }
        }
    };
}

bevy_utils::all_tuples!(impl_action_tuple, 1, 15, T);

impl IntoBoxedActions for BoxedAction {
    fn into_boxed_actions(
        self,
    ) -> impl DoubleEndedIterator<Item = BoxedAction> + ExactSizeIterator + Send + Debug + 'static
    {
        [self].into_iter()
    }
}

impl<const N: usize> IntoBoxedActions for [BoxedAction; N] {
    fn into_boxed_actions(
        self,
    ) -> impl DoubleEndedIterator<Item = BoxedAction> + ExactSizeIterator + Send + Debug + 'static
    {
        self.into_iter()
    }
}
