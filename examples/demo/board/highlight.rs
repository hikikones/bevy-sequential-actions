use bevy::{ecs::system::Command, prelude::*};

use crate::assets::*;

use super::*;

pub(super) struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Debug, Component)]
pub enum TileHighlightAction {
    Show(Vec<SquareCell>),
    ShowFocus(SquareCell),
    MoveFocus(IVec2),
    Clear,
    ClearFocus,
}

#[derive(Component)]
pub struct TileFocus;

#[derive(Component)]
struct Highlight;

pub trait TileHighlightExt {
    fn tile_highlight(&mut self, action: TileHighlightAction);
}

impl TileHighlightExt for World {
    fn tile_highlight(&mut self, action: TileHighlightAction) {
        match action {
            TileHighlightAction::Show(cells) => {
                let mesh = self.resource::<Meshes>().get(MeshName::Quad);
                let material = self.resource::<Materials>().get(MaterialName::Silver);
                let cell_size = self.resource::<Board>().cell_size();
                for cell in cells {
                    self.spawn()
                        .insert_bundle(PbrBundle {
                            mesh: mesh.clone(),
                            material: material.clone(),
                            transform: Transform {
                                translation: cell.as_point(cell_size) + Vec3::Y * 0.01,
                                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                                scale: Vec3::ONE * 0.95,
                            },
                            ..Default::default()
                        })
                        .insert(Highlight);
                }
            }
            TileHighlightAction::ShowFocus(cell) => {
                let mesh = self.resource::<Meshes>().get(MeshName::Quad);
                let material = self.resource::<Materials>().get(MaterialName::Silver);
                let cell_size = self.resource::<Board>().cell_size();
                self.spawn()
                    .insert_bundle(PbrBundle {
                        mesh,
                        material,
                        transform: Transform {
                            translation: cell.as_point(cell_size) + Vec3::Y * 0.01,
                            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                            scale: Vec3::ONE * 0.95,
                        },
                        ..Default::default()
                    })
                    .insert(TileFocus);
            }
            TileHighlightAction::MoveFocus(dpad) => {
                self.resource_scope(|world, board: Mut<Board>| {
                    let mut focus = world
                        .query_filtered::<&mut Transform, With<TileFocus>>()
                        .iter_mut(world)
                        .next()
                        .unwrap();
                    let current = board.get_cell(focus.translation);
                    let next = current + dpad;
                    if !board.is_cell_outside(next) && current != next {
                        focus.translation = next.as_point(board.cell_size()) + Vec3::Y * 0.01;
                    }
                });
            }
            TileHighlightAction::Clear => {
                let entities = self
                    .query_filtered::<Entity, With<Highlight>>()
                    .iter(self)
                    .collect::<Vec<_>>();
                for entity in entities {
                    self.entity_mut(entity).despawn_recursive();
                }
            }
            TileHighlightAction::ClearFocus => {
                let focus = self
                    .query_filtered::<Entity, With<TileFocus>>()
                    .iter(self)
                    .next()
                    .unwrap();
                self.entity_mut(focus).despawn_recursive();
            }
        }
    }
}

impl Command for TileHighlightAction {
    fn write(self, world: &mut World) {
        world.tile_highlight(self);
    }
}

impl TileHighlightExt for Commands<'_, '_> {
    fn tile_highlight(&mut self, action: TileHighlightAction) {
        self.add(action);
    }
}
