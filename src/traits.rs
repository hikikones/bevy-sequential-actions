use bevy_ecs::prelude::*;

use crate::*;

/// Conversion into an [`Action`].
pub trait IntoAction {
    fn into_boxed(self) -> Box<dyn Action>;
}

impl<T> IntoAction for T
where
    T: Action + 'static,
{
    fn into_boxed(self) -> Box<dyn Action> {
        Box::new(self)
    }
}

impl IntoAction for Box<dyn Action> {
    fn into_boxed(self) -> Box<dyn Action> {
        self
    }
}

/// Extension trait for `add_action` method on [`ActionCommands`] and [`Commands`].
pub trait AddActionExt {
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig);
}

/// Extension trait for `stop_action` method on [`ActionCommands`] and [`Commands`].
pub trait StopActionExt {
    fn stop_action(&mut self, actor: Entity);
}

/// Extension trait for `next_action` method on [`ActionCommands`] and [`Commands`].
pub trait NextActionExt {
    fn next_action(&mut self, actor: Entity);
}

/// Extension trait for `clear_actions` method on [`ActionCommands`] and [`Commands`].
pub trait ClearActionsExt {
    fn clear_actions(&mut self, actor: Entity);
}
