use bevy::prelude::*;

mod play;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::None)
            .add_plugin(play::MovementPlugin);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    None,
    Play,
    Observe,
}
