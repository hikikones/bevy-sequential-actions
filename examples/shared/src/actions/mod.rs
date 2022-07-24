use bevy::prelude::*;

pub mod command_action;
pub mod despawn_action;
pub mod move_action;
pub mod wait_action;

pub use command_action::CommandAction;
pub use despawn_action::DespawnAction;
pub use move_action::MoveAction;
pub use wait_action::WaitAction;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(wait_action::WaitActionPlugin)
            .add_plugin(move_action::MoveActionPlugin);
    }
}
