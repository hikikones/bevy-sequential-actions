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
///         // Action is finished.
///         commands.actions(entity).finish();
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
/// fn wait(mut wait_q: Query<(Entity, &mut Wait)>, time: Res<Time>, mut commands: Commands) {
///     for (entity, mut wait) in wait_q.iter_mut() {
///         wait.0 -= time.delta_seconds();
///         if wait.0 <= 0.0 {
///             // Action is finished.
///             commands.actions(entity).finish();
///         }
///     }
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an action is started.
    fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands);

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, entity: Entity, world: &mut World, reason: StopReason);
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
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn on_start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Bad
///         world.actions(entity).finish();
///
///         // Good
///         commands.actions(entity).finish();
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
    /// The type returned for building a list of actions.
    type Builder: ActionBuilder;

    /// Sets the current [`config`](AddConfig) for actions to be added.
    fn config(self, config: AddConfig) -> Self;

    /// Adds an [`action`](Action) to the queue with the current [`config`](AddConfig).
    fn add<T: IntoAction>(self, action: T) -> Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn next(self) -> Self;

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue.
    /// Current action is [`stopped`](Action::on_stop) as [`finished`](StopReason::Finished).
    fn finish(self) -> Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) as [`paused`](StopReason::Paused).
    fn pause(self) -> Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action) with specified [`reason`](StopReason).
    fn stop(self, reason: StopReason) -> Self;

    /// Clears the actions queue.
    /// Current [`action`](Action) is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    fn clear(self) -> Self;

    /// Build a list of actions.
    fn builder(self) -> Self::Builder;
}

/// Methods for building a list of actions.
pub trait ActionBuilder {
    /// The type that is returned after [`submit`](Self::submit) is called.
    type Modifier: ModifyActions;

    /// Sets the current [`config`](AddConfig) for actions to be pushed.
    fn config(self, config: AddConfig) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions will not be added to the queue until [`submit`](Self::submit) is called.
    fn push<T: IntoAction>(self, action: T) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submits the [`pushed`](Self::push) actions by consuming the list and adding them to the queue.
    fn submit(self) -> Self::Modifier;
}
