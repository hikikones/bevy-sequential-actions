use bevy::prelude::*;

use bevy_sequential_actions::*;

use crate::{assets::*, bevy_actions::*, board::*, camera::*, input::*, player::Player};

use super::GameState;

pub(super) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Play).with_system(on_enter_play))
            .add_system_set(
                SystemSet::on_update(GameState::Play)
                    .with_system(on_focus_added)
                    .with_system(on_focus_changed)
                    .with_system(on_input),
            )
            .add_system_set(SystemSet::on_exit(GameState::Play).with_system(on_exit_play));
    }
}

#[derive(Default, Component)]
struct IsValidTarget(bool);

fn on_enter_play(
    player_q: Query<(Entity, &Transform), With<Player>>,
    mut commands: Commands,
    board: Res<Board>,
) {
    println!("\n---------- Play ----------");
    println!("Use arrows to move.");
    println!("Press 'Enter' to confirm.\n");

    let player = player_q.single();
    let start = board.get_cell(player.1.translation);
    commands.tile_highlight(TileHighlightAction::ShowFocus(start));
    commands.spawn().insert(IsValidTarget::default());
}

fn on_focus_added(focus_q: Query<Entity, Added<TileFocus>>, mut commands: Commands) {
    for focus in focus_q.iter() {
        commands.camera(CameraCommand::Follow(focus));
    }
}

fn on_focus_changed(
    focus_q: Query<(Entity, &Transform), (With<TileFocus>, Changed<Transform>)>,
    player_q: Query<(Entity, &Transform), With<Player>>,
    mut valid_target_q: Query<&mut IsValidTarget>,
    board: Res<Board>,
    materials: Res<Materials>,
    mut commands: Commands,
) {
    for focus in focus_q.iter() {
        commands.tile_highlight(TileHighlightAction::Clear);

        let player = player_q.single();
        let start = board.get_cell(player.1.translation);
        let target = board.get_cell(focus.1.translation);
        let mut valid_target = valid_target_q.single_mut();

        if start == target {
            valid_target.0 = false;
            commands
                .entity(focus.0)
                .insert(materials.get(MaterialName::Red));
            return;
        }

        let path = AStar::new(&**board)
            .find_path(start, target)
            .unwrap_or(vec![]);

        commands.tile_highlight(TileHighlightAction::Show(path));

        let tile = board.get_tile(target);
        let material = match tile {
            Tile::None => {
                valid_target.0 = false;
                materials.get(MaterialName::Red)
            }
            _ => {
                valid_target.0 = true;
                materials.get(MaterialName::SeaGreen)
            }
        };
        commands.entity(focus.0).insert(material);
    }
}

fn on_input(
    mut input_evr: EventReader<InputEvent>,
    valid_target_q: Query<&IsValidTarget>,
    player_q: Query<Entity, With<Player>>,
    focus_q: Query<(Entity, &Transform), With<TileFocus>>,
    board: Res<Board>,
    mut commands: Commands,
) {
    for input in input_evr.iter() {
        match input {
            InputEvent::Enter => {
                if !valid_target_q.single().0 {
                    return;
                }

                let player = player_q.single();
                let focus = focus_q.single();
                let cell = board.get_cell(focus.1.translation);
                commands
                    .action_builder(player, AddConfig::default())
                    .add(SetStateAction::new(GameState::Observe))
                    .add(CameraAction::Pan(PanTarget::Entity(player), 1.0))
                    .add(CameraAction::Follow(player))
                    .add(MoveAction::new(cell))
                    .add(TileEventAction)
                    .add(SetStateAction::new(GameState::Play))
                    .submit();
            }
            InputEvent::Dpad(dpad) => {
                commands.tile_highlight(TileHighlightAction::MoveFocus(*dpad));
            }
        }
    }
}

fn on_exit_play(valid_target_q: Query<Entity, With<IsValidTarget>>, mut commands: Commands) {
    commands.entity(valid_target_q.single()).despawn();
    commands.tile_highlight(TileHighlightAction::Clear);
    commands.tile_highlight(TileHighlightAction::ClearFocus);
}
