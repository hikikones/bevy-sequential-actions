use bevy::prelude::*;

mod camera_action;
mod lerp_action;
mod move_action;
mod set_state_action;
mod tile_event_action;
mod tile_trap_action;
mod wait_action;

pub use camera_action::{CameraAction, PanTarget};
pub use lerp_action::{LerpAction, LerpType};
pub use move_action::MoveAction;
pub use set_state_action::SetStateAction;
pub use tile_event_action::TileEventAction;
pub use tile_trap_action::TileTrapAction;
pub use wait_action::WaitAction;

pub(super) struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(wait_action::WaitActionPlugin)
            .add_plugin(lerp_action::LerpActionPlugin)
            .add_plugin(move_action::MoveActionPlugin)
            .add_plugin(camera_action::CameraActionPlugin)
            .add_plugin(tile_event_action::TileEventActionPlugin)
            .add_plugin(tile_trap_action::TileTrapActionPlugin)
            .add_plugin(set_state_action::SetStateActionPlugin);
    }
}
