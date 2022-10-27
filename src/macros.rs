#[macro_export]
macro_rules! actions {
    ( $( $x:expr ),* $(,)? ) => {
        [ $( $crate::IntoBoxedAction::into_boxed($x) ),* ].into_iter()
    };
}
