/// Helper macro for creating a collection of boxed actions.
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy_sequential_actions::*;
/// # use shared::actions::QuitAction;
/// #
/// # let action_a = QuitAction;
/// # let action_b = QuitAction;
/// #
/// let actions: std::array::IntoIter<BoxedAction, 3> = actions![
///         action_a,
///         action_b,
///         |agent: Entity, world: &mut World, commands: &mut ActionCommands| {
///             // on_start
///         },
///     ];
/// ```
#[macro_export]
macro_rules! actions {
    ( $( $action:expr ),* $(,)? ) => {
        [ $( ::core::convert::Into::<::std::boxed::Box<dyn $crate::Action>>::into($action) ),* ].into_iter()
    }
}
