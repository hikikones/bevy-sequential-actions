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
        Box::new( [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ].into_iter() )
    }
}

// #[macro_export]
// macro_rules! actions_2d {
//     ( $( [ $( $d:expr ),* $(,)? ] ),* $(,)? ) => {
//         Box::new([
//             $(
//                 Box::new([ $( $crate::IntoBoxedAction::into_boxed($d) ),* ]) as Box<[_]>,
//             )*
//         ])
//     }
// }

#[macro_export]
macro_rules! sequential_actions {
    ( $( $x:expr ),* $(,)? ) => {
        $crate::ActionType::Sequence(actions![$( $x )*])
    };
}

#[macro_export]
macro_rules! parallel_actions {
    ( $( $x:expr ),* $(,)? ) => {
        $crate::ActionType::Parallel(actions![$( $x )*])
    };
}

// #[macro_export]
// macro_rules! linked_actions {
//     ( $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ) => {
//         $crate::ActionType::Linked(Box::new([
//             $(
//                 // actions![$( $x )*]
//                 Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
//             )*
//         ]))
//     }
// }

#[macro_export]
macro_rules! linked_actions {
    ( $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ) => {
        $crate::ActionType::Linked(Box::new([
            $(
                // Box::<[dyn ExactSizeIterator<Item = Empty>]>::new( acts![$( $x ),*].into_iter() ),
                // Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
                Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
            )*
        ].into_iter()))
    }
}

macro_rules! acts {
    ( $( $x:expr ),* $(,)? ) => {
        [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]
    }
}

// macro_rules! linked {
//     ( $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ) => {
//         Box::new([
//             $(
//                 // Box::<[dyn ExactSizeIterator<Item = Empty>]>::new( acts![$( $x ),*].into_iter() ),
//                 // Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
//                 Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
//             )*
//         ].into_iter())
//     }
// }

enum Yoyo {
    Linked(Box<dyn ExactSizeIterator<Item = Box<[BoxedAction]>>>),
}

macro_rules! linked {
    ( $( [ $( $x:expr ),* $(,)? ] ),* $(,)? ) => {
        Yoyo::Linked(Box::new([
            $(
                // Box::<[dyn ExactSizeIterator<Item = Empty>]>::new( acts![$( $x ),*].into_iter() ),
                // Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
                Box::new([ $( $crate::IntoBoxedAction::into_boxed($x) ),* ]) as Box<[_]>,
            )*
        ].into_iter()))
    }
}

fn test() {
    // let a = linked!([Empty], [Empty, Empty]);
    // let b = a.filter(|x| !x.is_empty()).collect::<Box<[_]>>();

    // let a = Box::new([Box::new([Empty]), Box::new([Empty, Empty]) as Box<[_]>].into_iter());
    // let b = a.filter(|x| !x.is_empty()).collect::<Box<[_]>>();

    let a: Box<dyn ExactSizeIterator<Item = Box<[Empty]>>> =
        Box::new([Box::new([Empty]), Box::new([Empty, Empty]) as Box<[_]>].into_iter());

    // let a = Box::<Box<dyn Iterator<Item = Box<dyn Iterator<Item = Empty>>>>>::new([
    //     (Box::new([Empty]) as Box<[_]>).into_iter(),
    //     (Box::new([Empty, Empty]) as Box<[_]>).into_iter(),
    // ]);
    // let a: Box<[Box<dyn ExactSizeIterator<Item = Empty>>; 2]> = Box::new([
    //     Box::new([Empty].into_iter()),
    //     Box::new([Empty, Empty].into_iter()),
    // ]);

    let b = linked![[Empty], [Empty, Empty]];
}

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;

struct Empty;
impl Action for Empty {
    fn on_start(&mut self, agent: Entity, world: &mut World, commands: &mut crate::ActionCommands) {
        todo!()
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: crate::StopReason) {
        todo!()
    }
}
