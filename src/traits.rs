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
///     fn on_finish(&mut self, entity: Entity, world: &mut World) {}
///     fn on_cancel(&mut self, entity: Entity, world: &mut World) {}
///     fn on_stop(&mut self, entity: Entity, world: &mut World) {}
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
///         world
///             .entity_mut(entity)
///             .insert(Wait(self.current.unwrap_or(self.duration)));
///     }
///
///     fn on_finish(&mut self, entity: Entity, world: &mut World) {
///         world.entity_mut(entity).remove::<Wait>();
///         self.current = None;
///     }
///
///     fn on_cancel(&mut self, entity: Entity, world: &mut World) {
///         self.on_finish(entity, world);
///     }
///
///     fn on_stop(&mut self, entity: Entity, world: &mut World) {
///         let wait = world.entity_mut(entity).remove::<Wait>().unwrap();
///         self.current = Some(wait.0);
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

    /// The method that is called when an action is finished.
    fn on_finish(&mut self, entity: Entity, world: &mut World);

    /// The method that is called when an action is canceled.
    fn on_cancel(&mut self, entity: Entity, world: &mut World);

    /// The method that is called when an action is stopped.
    fn on_stop(&mut self, entity: Entity, world: &mut World);
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
///     fn on_finish(&mut self, entity: Entity, world: &mut World) {}
///     fn on_cancel(&mut self, entity: Entity, world: &mut World) {}
///     fn on_stop(&mut self, entity: Entity, world: &mut World) {}
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
    fn add(self, action: impl IntoAction) -> Self;

    /// [`Starts`](Action::on_start) the next action in the queue.
    /// Current action is [`canceled`](Action::on_cancel) and removed from the queue.
    fn next(self) -> Self;

    /// [`Starts`](Action::on_start) the next action in the queue.
    /// Current action is [`finished`](Action::on_cancel) and removed from the queue.
    fn finish(self) -> Self;

    /// [`Stops`](Action::on_stop) the current [`action`](Action).
    /// A stopped action is not removed from the queue.
    fn stop(self) -> Self;

    /// Clears the actions queue.
    /// Current action is [`canceled`](Action::on_cancel).
    fn clear(self) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions __will not__ be added to the queue until [`submit`](Self::submit) is called.
    fn push(self, action: impl IntoAction) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submits the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
