/// Helper macro for creating an array of boxed actions.
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_sequential_actions::*;
/// #
/// # struct EmptyAction;
/// # impl Action for EmptyAction {
/// #   fn is_finished(&self, _a: Entity, _w: &World) -> Finished { true.into() }
/// #   fn on_start(&mut self, _a: Entity, _w: &mut World) -> Finished { true.into() }
/// #   fn on_stop(&mut self, _a: Entity, _w: &mut World, _r: StopReason) {}
/// # }
/// #
/// # let action_a = EmptyAction;
/// # let action_b = EmptyAction;
/// #
/// let actions: [Box<dyn Action>; 3] = actions![
///         action_a,
///         action_b,
///         |agent: Entity, world: &mut World| -> Finished {
///             // on_start
///             Finished(true)
///         },
///     ];
/// ```
#[macro_export]
macro_rules! actions {
    ( $( $action:expr ),* $(,)? ) => {
        [ $( ::core::convert::Into::<::std::boxed::Box<dyn $crate::Action>>::into($action) ),* ]
    }
}
