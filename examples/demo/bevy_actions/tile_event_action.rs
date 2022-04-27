use bevy::prelude::*;

use crate::{board::*, camera::*};
use bevy_sequential_actions::*;

use super::*;

pub(super) struct TileEventActionPlugin;

impl Plugin for TileEventActionPlugin {
    fn build(&self, _app: &mut App) {}
}

pub struct TileEventAction;

impl Action for TileEventAction {
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
        let pos = world.get::<Transform>(actor).unwrap().translation;
        let board = world.resource::<Board>();
        let cell = board.get_cell(pos);
        let tile = board.get_tile(cell);

        if Tile::Event == *tile {
            let camera = world
                .query_filtered::<Entity, With<CameraMain>>()
                .iter(world)
                .next()
                .unwrap();

            commands
                .action_builder(
                    actor,
                    AddConfig {
                        order: AddOrder::Front,
                        start: false,
                        repeat: false,
                    },
                )
                .add(CameraAction::Pan(PanTarget::Entity(actor), 0.5))
                .add(LerpAction::new(
                    camera,
                    LerpType::Position(CAMERA_OFFSET * 0.5),
                    1.0,
                ))
                .add(LerpAction::new(actor, LerpType::Face(camera), 1.0))
                .add(WaitAction::new(1.0))
                .add(LerpAction::new(
                    camera,
                    LerpType::Position(CAMERA_OFFSET),
                    1.0,
                ))
                .reverse()
                .submit();
        }

        commands.next_action(actor);
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}
