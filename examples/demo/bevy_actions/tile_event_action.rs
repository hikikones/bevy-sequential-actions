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
    fn start(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
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
                .action(actor)
                .config(AddConfig {
                    order: AddOrder::Front,
                    start: false,
                    repeat: false,
                })
                .push(CameraAction::Pan(PanTarget::Entity(actor), 0.5))
                .push(LerpAction::new(
                    camera,
                    LerpType::Position(CAMERA_OFFSET * 0.5),
                    1.0,
                ))
                .push(LerpAction::new(actor, LerpType::Face(camera), 1.0))
                .push(WaitAction::new(1.0))
                .push(LerpAction::new(
                    camera,
                    LerpType::Position(CAMERA_OFFSET),
                    1.0,
                ))
                .reverse()
                .submit();
        }

        commands.action(actor).next();
    }

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}
