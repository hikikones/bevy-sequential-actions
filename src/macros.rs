/// Helper macro for creating a collection of boxed actions.
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// # use shared::actions::*;
/// #
/// let actions: std::array::IntoIter<BoxedAction, 4> = actions![
///         QuitAction,
///         DespawnAction,
///         WaitAction::new(1.0),
///         |agent: Entity, world: &mut World, commands: &mut ActionCommands| {
///             // on_start
///         },
///     ];
/// ```
#[macro_export]
macro_rules! actions {
    ( $( $x:expr ),* $(,)? ) => {
        Box::new( [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ].into_iter() )
    };
}

#[macro_export]
macro_rules! actions_2d {
    [ $( [ $( $d:expr ),* $(,)? ] ),* $(,)? ] => {
        Box::new([
            $(
                Box::new([ $( $crate::IntoBoxedAction::into_boxed($d) ),* ]) as Box<[_]>,
            )*
        ])
    }
}
