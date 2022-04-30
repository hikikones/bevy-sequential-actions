#![allow(dead_code)]

use bevy::prelude::*;

mod assets;
mod bevy_actions;
mod bevy_extensions;
mod bevy_grid;
mod board;
mod camera;
mod input;
mod player;

use assets::*;
use bevy_actions::*;
use bevy_sequential_actions::*;
use board::*;
use camera::*;
use input::*;
use player::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    None,
    Play,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Demo | Bevy Sequential Actions".into(),
            width: 1024.0,
            height: 720.0,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.15, 0.3, 0.35)))
        .add_state(GameState::None)
        .add_plugins(DefaultPlugins)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(bevy_actions::ActionsPlugin)
        .add_plugin(board::BoardPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(input::InputPlugin)
        .add_startup_system_to_stage(StartupStage::PostStartup, start_game)
        .add_system_set(SystemSet::on_enter(GameState::Play).with_system(on_enter_play))
        .add_system_set(
            SystemSet::on_update(GameState::Play)
                .with_system(on_focus_added)
                .with_system(on_focus_changed)
                .with_system(on_input),
        )
        .add_system_set(SystemSet::on_exit(GameState::Play).with_system(on_exit_play))
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

fn start_game(mut game_state: ResMut<State<GameState>>) {
    game_state.set(GameState::Play).unwrap();
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

        let mut path = AStar::new(&**board)
            .search(start, target, EdgeWeight::Custom)
            .unwrap_or(vec![]);
        path.pop();

        commands.tile_highlight(TileHighlightAction::Show(path));

        let tile = board.get_tile(target);
        let material = match tile {
            Tile::Blocked => {
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
                    .push(SetStateAction::new(GameState::None))
                    .push(CameraAction::Pan(PanTarget::Entity(player), 1.0))
                    .push(CameraAction::Follow(player))
                    .push(MoveAction::new(cell))
                    .push(TileEventAction)
                    .push(SetStateAction::new(GameState::Play))
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
