use bevy::prelude::*;

use shared::playground::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlaygroundPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(player_q: Query<Entity, With<Player>>, mut commands: Commands) {
    let p = player_q.single();
    println!("{:?}", p);
}
