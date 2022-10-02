use bevy::{app::AppExit, ecs::event::Events};
use bevy_sequential_actions::*;

pub struct QuitAction;

impl Action for QuitAction {
    fn on_start(&mut self, state: &mut WorldState, _commands: &mut ActionCommands) {
        state.world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn on_stop(&mut self, _state: &mut WorldState, _reason: StopReason) {}
}
