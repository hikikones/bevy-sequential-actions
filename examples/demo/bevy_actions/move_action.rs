use bevy::prelude::*;

use bevy_sequential_actions::*;

use super::TileTrapAction;
use crate::{bevy_extensions::*, board::*};

pub(super) struct MoveActionPlugin;

impl Plugin for MoveActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_action).add_system(rotate);
    }
}

pub struct MoveAction(SquareCell);

impl MoveAction {
    pub fn new(target: SquareCell) -> Self {
        Self(target)
    }
}

impl Action for MoveAction {
    fn start(&mut self, actor: Entity, world: &mut World, _commands: &mut ActionCommands) {
        let board = world.get_resource::<Board>().unwrap();
        let start = board.get_cell(world.get::<Transform>(actor).unwrap().translation);
        let goal = self.0;
        let path = AStar::new(&**board)
            .search(start, goal, EdgeWeight::Custom)
            .unwrap()
            .iter()
            .map(|c| c.as_point(board.cell_size()))
            .collect();

        world.entity_mut(actor).insert_bundle(MoveBundle {
            path: Path(path),
            index: Index(0),
            speed: Speed(4.0),
        });
    }

    fn stop(&mut self, actor: Entity, world: &mut World) {
        world.entity_mut(actor).remove_bundle::<MoveBundle>();
    }
}

#[derive(Bundle)]
struct MoveBundle {
    path: Path,
    index: Index,
    speed: Speed,
}

#[derive(Component)]
struct Path(Box<[Vec3]>);

#[derive(Component)]
struct Index(usize);

#[derive(Component)]
struct Speed(f32);

fn move_action(
    mut q: Query<(Entity, &mut Transform, &mut Index, &Path, &Speed)>,
    time: Res<Time>,
    board: Res<Board>,
    mut commands: Commands,
) {
    for (actor, mut transform, mut index, path, speed) in q.iter_mut() {
        if transform.move_towards(path.0[index.0], speed.0 * time.delta_seconds()) {
            if index.0 == path.0.len() - 1 {
                commands.action(actor).next();
                continue;
            }

            if index.0 > 0 {
                let pos = transform.translation;
                let cell = board.get_cell(pos);
                let tile = board.get_tile(cell);

                if Tile::Trap == *tile {
                    commands
                        .action(actor)
                        .stop()
                        .config(AddConfig {
                            order: AddOrder::Front,
                            start: true,
                            repeat: false,
                        })
                        .add(TileTrapAction);
                    continue;
                }
            }

            index.0 += 1;
        }
    }
}

fn rotate(mut q: Query<(&mut Transform, &Index, &Path, &Speed)>, time: Res<Time>) {
    for (mut transform, index, path, speed) in q.iter_mut() {
        let dir = path.0[index.0] - transform.translation;

        if dir == Vec3::ZERO {
            continue;
        }

        transform.rotation = Quat::slerp(
            transform.rotation,
            Quat::look_rotation(dir, Vec3::Y),
            speed.0 * 2.0 * time.delta_seconds(),
        );
    }
}
