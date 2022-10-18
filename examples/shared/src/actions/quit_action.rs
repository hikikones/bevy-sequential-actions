use bevy::{
    app::AppExit,
    ecs::{event::Events, world::World},
};
use bevy_sequential_actions::*;

pub struct QuitAction;

impl Action for QuitAction {
    fn on_start(&mut self, _id: ActionIds, world: &mut World, _commands: &mut ActionCommands) {
        world.resource_mut::<Events<AppExit>>().send(AppExit);
    }

    fn on_stop(&mut self, _id: ActionIds, _world: &mut World, _reason: StopReason) {}
}
