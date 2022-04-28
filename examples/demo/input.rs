use bevy::prelude::*;

pub(super) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputEvent>().add_system(input);
    }
}

#[derive(Debug)]
pub enum InputEvent {
    Enter,
    Dpad(IVec2),
}

fn input(keyboard: Res<Input<KeyCode>>, mut input_evw: EventWriter<InputEvent>) {
    if keyboard.just_pressed(KeyCode::Return) {
        input_evw.send(InputEvent::Enter);
    } else if keyboard.just_pressed(KeyCode::Right) {
        input_evw.send(InputEvent::Dpad(IVec2::X));
    } else if keyboard.just_pressed(KeyCode::Left) {
        input_evw.send(InputEvent::Dpad(-IVec2::X));
    } else if keyboard.just_pressed(KeyCode::Up) {
        input_evw.send(InputEvent::Dpad(-IVec2::Y));
    } else if keyboard.just_pressed(KeyCode::Down) {
        input_evw.send(InputEvent::Dpad(IVec2::Y));
    }
}
