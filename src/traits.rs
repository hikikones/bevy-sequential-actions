use bevy_ecs::prelude::*;

use crate::*;

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

pub trait AddActionExt {
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig);
}

pub trait StopActionExt {
    fn stop_action(&mut self, actor: Entity);
}

pub trait NextActionExt {
    fn next_action(&mut self, actor: Entity);
}

pub trait ClearActionsExt {
    fn clear_actions(&mut self, actor: Entity);
}
