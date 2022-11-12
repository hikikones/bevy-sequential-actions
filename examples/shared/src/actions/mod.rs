use bevy::prelude::*;
use bevy_sequential_actions::*;

pub mod despawn_action;
pub mod lerp_action;
pub mod move_action;
pub mod quit_action;
pub mod rotate_action;
pub mod set_state_action;
pub mod wait_action;

pub use despawn_action::*;
pub use lerp_action::*;
pub use move_action::*;
pub use quit_action::*;
pub use rotate_action::*;
pub use set_state_action::*;
pub use wait_action::*;

use crate::extensions::RandomExt;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SequentialActionsPlugin)
            .add_plugin(WaitActionPlugin)
            .add_plugin(MoveActionPlugin)
            .add_plugin(RotateActionPlugin)
            .add_plugin(LerpActionPlugin);
    }
}

pub trait IntoValue<T = Self>
where
    Self: Send + Sync + 'static,
{
    fn value(&self) -> T;
}

impl IntoValue for f32 {
    fn value(&self) -> Self {
        *self
    }
}

impl IntoValue for Vec3 {
    fn value(&self) -> Self {
        *self
    }
}

#[derive(Clone, Copy)]
pub struct Random<T>
where
    T: RandomExt,
{
    min: T,
    max: T,
}

impl<T> Random<T>
where
    T: RandomExt,
{
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> IntoValue<T> for Random<T>
where
    T: Clone + Copy + RandomExt + IntoValue<T>,
{
    fn value(&self) -> T {
        T::random(self.min, self.max)
    }
}
