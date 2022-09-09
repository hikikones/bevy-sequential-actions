use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::{schedule::SystemStage, system::Query};

use crate::*;

pub struct SequentialActionsPlugin;

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::PostUpdate,
            CHECK_ACTIONS_STAGE,
            SystemStage::parallel(),
        )
        .add_system_to_stage(CHECK_ACTIONS_STAGE, check_actions);
    }
}

pub(super) const CHECK_ACTIONS_STAGE: &str = "check_actions";

pub(super) fn check_actions(
    mut q: Query<
        (Entity, &CurrentAction, &mut ActionFinished),
        (Changed<ActionFinished>, With<ActionMarker>),
    >,
    mut commands: Commands,
) {
    for (entity, current_action, mut finished) in q.iter_mut() {
        if let Some((action_type, _)) = &current_action.0 {
            let is_finished = match action_type {
                ActionType::Single(_) => finished.count == 1,
                ActionType::Many(actions) => finished.count == actions.len() as u32,
            };

            if is_finished {
                commands.actions(entity).finish();
            }

            dbg!(finished.count);
            finished.count = 0;
        }
    }
}

fn reset_action_status(mut q: Query<&mut ActionFinished, With<ActionMarker>>) {
    for mut finished in q.iter_mut() {
        finished.count = 0;
    }
}
