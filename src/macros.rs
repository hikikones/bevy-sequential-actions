/// Helper macro for creating a collection of boxed actions.
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// # use shared::actions::*;
/// #
/// fn setup(mut commands: Commands) {
/// #   let agent = commands.spawn_bundle(ActionsBundle::default()).id();
/// #
///     commands
///         .actions(agent)
///         .add_many(
///             ExecutionMode::Parallel,
///             actions![
///                 QuitAction,
///                 DespawnAction,
///                 WaitAction::new(1.0),
///                 |agent: Entity, world: &mut World, commands: &mut ActionCommands | {
///                     // on_start
///                 },
///             ]
///         );
/// }
/// ```
#[macro_export]
macro_rules! actions {
    ( $( $x:expr ),* $(,)? ) => {
        [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ].into_iter()
    };
}
