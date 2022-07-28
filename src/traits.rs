use crate::*;

/// The trait that all actions must implement.
///
/// All actions must declare when they are done.
/// This is done by calling [`next`](ModifyActions::next)
/// from either [`ActionCommands`] or [`Commands`].
///
/// # Examples
///
/// #### Empty Action
///
/// An action that does nothing.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// # fn main() {}
/// #
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn start(&mut self, state: StartState, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Action is finished, issue next.
///         commands.actions(entity).finish();
///     }
///
///     fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {}
/// }
/// ```
///
/// #### Wait Action
///
/// An action that waits a specified time in seconds.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// # use shared::actions::QuitAction;
/// #
/// # fn main() {
/// #     App::new()
/// #         .add_plugins(MinimalPlugins)
/// #         .add_startup_system(setup)
/// #         .add_system(wait)
/// #         .run();
/// # }
/// #
/// # fn setup(mut commands: Commands) {
/// #     let entity = commands.spawn_bundle(ActionsBundle::default()).id();
/// #     commands.actions(entity).add(WaitAction::new(0.0)).add(QuitAction);
/// # }
/// pub struct WaitAction {
///     duration: f32,
///     current: f32,
/// }
///
/// # impl WaitAction {
/// #     pub fn new(seconds: f32) -> Self {
/// #         Self {
/// #             duration: seconds,
/// #             current: 0.0,
/// #         }
/// #     }
/// # }
/// #
/// impl Action for WaitAction {
///     fn start(
///         &mut self,
///         state: StartState,
///         entity: Entity,
///         world: &mut World,
///         commands: &mut ActionCommands,
///     ) {
///         match state {
///             StartState::Init => {
///                 world.entity_mut(entity).insert(Wait(self.duration));
///             }
///             StartState::Resume => {
///                 world.entity_mut(entity).insert(Wait(self.current));
///             }
///         }
///     }
///
///     fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {
///         match reason {
///             StopReason::Completed | StopReason::Canceled => {
///                 world.entity_mut(entity).remove::<Wait>();
///             }
///             StopReason::Paused => {
///                 self.current = world.entity_mut(entity).remove::<Wait>().unwrap().0;
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
///             commands.actions(entity).finish();
///         }
///     }
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an action is started.
    fn start(
        &mut self,
        state: StartState,
        entity: Entity,
        world: &mut World,
        commands: &mut ActionCommands,
    );

    /// The method that is called when an action is stopped.
    fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World);

    // /// The method that is called when an action is finished.
    // fn finish(&mut self, entity: Entity, world: &mut World);

    // /// The method that is called when an action is canceled.
    // fn cancel(&mut self, entity: Entity, world: &mut World) {
    //     self.finish(entity, world);
    // }

    // /// The method that is called when an action is paused.
    // fn pause(&mut self, entity: Entity, world: &mut World) {
    //     self.finish(entity, world);
    // }

    // /// The method that is called when an action is resumed.
    // fn resume(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
    //     self.start(entity, world, commands);
    // }
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
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// # fn main() {}
/// #
/// struct EmptyAction;
/// impl Action for EmptyAction {
///     fn start(&mut self, state: StartState, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Bad
///         world.actions(entity).finish();
///
///         // Good
///         commands.actions(entity).finish();
///     }
///
///     fn stop(&mut self, reason: StopReason, entity: Entity, world: &mut World) {}
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

    /// [`Stops`](Action::stop) the current action as [`canceled`](StopReason::Canceled),
    /// and [`starts`](Action::start) the next action in the queue.
    fn next(self) -> Self;

    /// [`Stops`](Action::stop) the current action as [`completed`](StopReason::Completed),
    /// and [`starts`](Action::start) the next action in the queue.
    fn finish(self) -> Self;

    /// [`Stops`](Action::stop) the current [`action`](Action) with specified [`reason`](StopReason).
    fn stop(self, reason: StopReason) -> Self;

    /// Clears the actions queue.
    /// [`Stops`](Action::stop) the current action as [`canceled`](StopReason::Canceled).
    fn clear(self) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions __will not__ be added to the queue until [`submit`](Self::submit) is called.
    fn push(self, action: impl IntoAction) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submits the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
