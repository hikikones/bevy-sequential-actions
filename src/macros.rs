/// Helper macro for creating a collection of boxed actions.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// # use shared::actions::*;
/// #
/// let actions = actions![
///         QuitAction,
///         DespawnAction,
///         WaitAction::new(1.0),
///         |agent: Entity, world: &mut World, commands: &mut ActionCommands| {
///             // on_start
///         },
///     ];
///
/// assert_eq!(actions.count(), 4);
/// ```
#[macro_export]
macro_rules! actions {
    ( $( $x:expr ),* $(,)? ) => {
        [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ].into_iter()
    };
}
