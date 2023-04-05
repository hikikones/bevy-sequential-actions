/// Helper macro for creating a collection of boxed actions.
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// # struct EmptyAction;
/// # impl Action for EmptyAction {
/// #   fn is_finished(&self, _a: Entity, _w: &World) -> bool { true }
/// #   fn on_start(&mut self, _a: Entity, _w: &mut World) {}
/// #   fn on_stop(&mut self, _a: Entity, _w: &mut World, _r: StopReason) {}
/// # }
/// #
/// # let action_a = EmptyAction;
/// # let action_b = EmptyAction;
/// #
/// let actions: std::array::IntoIter<Box<dyn Action>, 3> = actions![
///         action_a,
///         action_b,
///         |agent: Entity, world: &mut World| {
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
