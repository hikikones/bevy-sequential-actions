use bevy::prelude::*;

pub mod despawn_action;
pub mod lerp_action;
pub mod move_action;
pub mod quit_action;
pub mod rotate_action;
pub mod wait_action;

pub use despawn_action::*;
pub use lerp_action::*;
pub use move_action::*;
pub use quit_action::*;
pub use rotate_action::*;
pub use wait_action::*;

use crate::extensions::RandomExt;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_sequential_actions::SequentialActionsPlugin)
            .add_plugin(WaitActionPlugin)
            .add_plugin(MoveActionPlugin)
            .add_plugin(RotateActionPlugin)
            .add_plugin(LerpActionPlugin);
    }
}

// pub trait MyValue
// where
//     Self: Send + Sync + 'static,
// {
//     fn value(self) -> Self;
// }

// impl MyValue for f32 {
//     fn value(self) -> Self {
//         self
//     }
// }

pub trait IntoValue<T = Self>
where
    Self: Send + Sync + 'static,
    T: Copy + Clone,
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

impl IntoValue for Quat {
    fn value(&self) -> Self {
        *self
    }
}

pub struct Random<T>
where
    T: RandomExt,
    T::Bound: Clone + Copy,
{
    min: T::Bound,
    max: T::Bound,
}

impl<T> Random<T>
where
    T: RandomExt,
    T::Bound: Clone + Copy,
{
    pub fn new(min: T::Bound, max: T::Bound) -> Self {
        Self { min, max }
    }
}

impl<T> IntoValue<T> for Random<T>
where
    T: RandomExt + IntoValue<T> + Clone + Copy,
    T::Bound: Clone + Copy + Send + Sync,
{
    fn value(&self) -> T {
        T::random(self.min, self.max)
    }
}
