use bevy::prelude::*;
use bevy_sequential_actions::SequentialActionsPlugin;

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
