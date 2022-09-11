use crate::*;

/// The trait that all actions must implement.
///
/// All actions must declare when they are finished.
/// This is done by calling [`finish`](ModifyActions::finish)
/// from either [`ActionCommands`] or [`Commands`].
///
/// # Examples
///
/// #### Empty Action
///
/// An action that does nothing.
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Issue next.
///         commands.actions(entity).next();
///     }
///
///     fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {}
/// }
/// ```
///
/// #### Wait Action
///
/// An action that waits a specified time in seconds.
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// pub struct WaitAction {
///     duration: f32,
///     current: Option<f32>,
/// }
///
/// impl Action for WaitAction {
///     fn on_start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
///         let duration = self.current.unwrap_or(self.duration);
///         world.entity_mut(entity).insert(Wait(duration));
///     }
///
///     fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
///         match reason {
///             StopReason::Finished | StopReason::Canceled => {
///                 world.entity_mut(entity).remove::<Wait>();
///                 self.current = None;
///             }
///             StopReason::Paused => {
///                 let wait = world.entity_mut(entity).remove::<Wait>().unwrap();
///                 self.current = Some(wait.0);
///             }
///         }
///     }
/// }
///
/// #[derive(Component)]
/// struct Wait(f32);
///
/// fn wait(mut wait_q: Query<(&mut Wait, &mut ActionFinished)>, time: Res<Time>) {
///     for (mut wait, mut finished) in wait_q.iter_mut() {
///         wait.0 -= time.delta_seconds();
///         if wait.0 <= 0.0 {
///             // Action is finished.
///             finished.confirm();
///         }
///     }
/// }
/// ```
pub trait Action: Send + Sync + 'static {
    /// The method that is called when an action is started.
    fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands);

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason);
}

impl<Start> Action for Start
where
    Start: FnMut(Entity, &mut World, &mut ActionCommands) + Send + Sync + 'static,
{
    fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
        (self)(entity, world, commands);
    }

    fn on_stop(&mut self, _entity: Entity, _world: &mut World, _reason: StopReason) {}
}

impl<Start, Stop> Action for (Start, Stop)
where
    Start: FnMut(Entity, &mut World, &mut ActionCommands) + Send + Sync + 'static,
    Stop: FnMut(Entity, &mut World, StopReason) + Send + Sync + 'static,
{
    fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
        (self.0)(entity, world, commands);
    }

    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {
        (self.1)(entity, world, reason);
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
///     fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Bad
///         world.actions(entity).next();
///
///         // Good
///         commands.actions(entity).next();
///
///         // Also good
///         commands.actions(entity).custom(move |w: &mut World| {
///             w.actions(entity).next();
///         });
///
///         // Also good if you want to mark it as finished
///         world.get_mut::<ActionFinished>(entity).unwrap().confirm();
///     }
///
///     fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason) {}
/// }
///```
pub trait ActionsProxy<'a> {
    /// The type returned for modifying actions.
    type Modifier: ModifyActions;

    /// Returns [`Self::Modifier`] for specified [`Entity`].
    fn actions(&'a mut self, entity: Entity) -> Self::Modifier;
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
