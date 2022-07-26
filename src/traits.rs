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
///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Action is finished, issue next.
///         commands.actions(entity).next();
///     }
///
///     fn finish(&mut self, entity: Entity, world: &mut World) {}
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
/// #     commands.actions(entity).add(WaitAction(0.0)).add(QuitAction);
/// # }
/// struct WaitAction(f32);
///
/// impl Action for WaitAction {
///     fn start(&mut self, entity: Entity, world: &mut World, _commands: &mut ActionCommands) {
///         world.entity_mut(entity).insert(Wait(self.0));
///     }
///
///     fn finish(&mut self, entity: Entity, world: &mut World) {
///         world.entity_mut(entity).remove::<Wait>();
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
///             // Action is finished, issue next.
///             commands.actions(entity).next();
///         }
///     }
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an action is started.
    fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands);

    /// The method that is called when an action is finished.
    fn finish(&mut self, entity: Entity, world: &mut World);
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
///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Bad
///         world.actions(entity).next();
///
///         // Good
///         commands.actions(entity).next();
///     }
///
///     fn finish(&mut self, entity: Entity, world: &mut World) {}
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

    /// Starts the next [`action`](Action) in the queue by [`stopping`](Action::stop) the currently running action,
    /// and [`starting`](Action::start) the next action in the queue list.
    fn next(self) -> Self;

    /// [`Finish`](Action::finish) the currently running [`action`](Action)
    /// by removing it from the queue and [`starting`](Action::start) the next one.
    fn finish(self) -> Self;

    /// [`Stops`](Action::stop) the currently running [`action`](Action), and clears any remaining.
    fn clear(self) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions __will not__ be added to the queue until [`submit`](Self::submit) is called.
    fn push(self, action: impl IntoAction) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submits the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
