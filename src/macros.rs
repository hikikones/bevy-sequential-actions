use crate::{Action, BoxedAction};

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
        [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]
    }
}

#[macro_export]
macro_rules! sequential_actions {
    ( $( $x:expr ),* $(,)? ) => {
        $crate::ActionType::Sequence(
            Box::new( actions![$( $x ),*].into_iter() )
        )
    };
}

#[macro_export]
macro_rules! parallel_actions {
    ( $( $x:expr ),* $(,)? ) => {
        $crate::ActionType::Parallel(
            Box::new( actions![$( $x ),*].into_iter() )
        )
    };
}

#[macro_export]
macro_rules! linked_actions {
    ( $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ) => {
        $crate::ActionType::Linked(Box::new([
            $(
                Box::new( actions![$( $x ),*] ) as Box<[_]>,
            )*
        ].into_iter()))
    }
}
