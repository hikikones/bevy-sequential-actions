use bevy::prelude::*;

mod countdown;
mod parallel;
mod print;
mod repeat;
mod sequence;

pub use countdown::*;
pub use parallel::*;
pub use print::*;
pub use repeat::*;
pub use sequence::*;

pub struct SharedActionsPlugin;

impl Plugin for SharedActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(countdown::CountdownActionPlugin);
    }
}
