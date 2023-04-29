/// Helper macro for creating an array of boxed actions.
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// # struct EmptyAction;
/// # impl Action for EmptyAction {
/// #   fn is_finished(&self, _a: Entity, _w: &World) -> bool { true }
/// #   fn on_start(&mut self, _a: Entity, _w: &mut World) -> bool { true }
/// #   fn on_stop(&mut self, _a: Entity, _w: &mut World, _r: StopReason) {}
/// # }
/// #
/// # let action_a = EmptyAction;
/// # let action_b = EmptyAction;
/// #
/// let actions: [Box<dyn Action>; 3] = actions![
///         action_a,
///         action_b,
///         |agent: Entity, world: &mut World| -> bool {
///             // on_start
///             true
///         },
///     ];
/// ```
#[macro_export]
macro_rules! actions {
    ( $( $action:expr ),* $(,)? ) => {
        [ $( ::core::convert::Into::<::std::boxed::Box<dyn $crate::Action>>::into($action) ),* ]
    }
}
