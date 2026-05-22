use bevy::prelude::*;

mod parallel;

pub use parallel::*;

pub struct SharedActionsPlugin;

impl Plugin for SharedActionsPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}
