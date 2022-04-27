use bevy::prelude::*;

use crate::{board::*, camera::*};
use bevy_sequential_actions::*;

use super::{LerpAction, LerpType};

pub(super) struct CameraActionPlugin;

impl Plugin for CameraActionPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Clone, Copy)]
pub enum CameraAction {
    Follow(Entity),
    Pan(PanTarget, f32),
}

#[derive(Clone, Copy)]
pub enum PanTarget {
    Entity(Entity),
    Cell(SquareCell),
    Position(Vec3),
}

impl Action for CameraAction {
    fn add(&mut self, actor: Entity, world: &mut World, commands: &mut ActionCommands) {
        match *self {
            CameraAction::Follow(target) => {
                world.camera(CameraCommand::Follow(target));
                commands.next_action(actor);
            }
            CameraAction::Pan(target, duration) => {
                world.camera(CameraCommand::Stop);

                let target_pos = match target {
                    PanTarget::Entity(entity) => {
                        world.get::<Transform>(entity).unwrap().translation
                    }
                    PanTarget::Cell(cell) => {
                        let board = world.get_resource::<Board>().unwrap();
                        cell.as_point(board.cell_size())
                    }
                    PanTarget::Position(pos) => pos,
                };

                let camera = world
                    .query_filtered::<Entity, With<CameraPivot>>()
                    .iter(world)
                    .next()
                    .unwrap();

                let lerp = LerpAction::new(camera, LerpType::Position(target_pos), duration);

                commands.add_action(
                    actor,
                    lerp,
                    AddConfig {
                        order: AddOrder::Front,
                        start: false,
                        repeat: false,
                    },
                );
                commands.next_action(actor);
            }
        }
    }

    fn remove(&mut self, _actor: Entity, _world: &mut World) {}

    fn stop(&mut self, _actor: Entity, _world: &mut World) {}
}
