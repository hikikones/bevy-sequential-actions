use bevy::prelude::*;

pub mod command_action;
pub mod despawn_action;
pub mod move_action;
pub mod quit_action;
pub mod wait_action;

pub use command_action::*;
pub use despawn_action::*;
pub use move_action::*;
pub use quit_action::*;
pub use wait_action::*;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WaitActionPlugin)
            .add_plugin(MoveActionPlugin);
    }
}
