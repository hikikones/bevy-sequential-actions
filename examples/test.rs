use bevy::prelude::*;

use shared::playground::PlaygroundPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlaygroundPlugin)
        .run();
}
