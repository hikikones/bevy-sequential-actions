use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::{schedule::SystemStage, system::Query};

use crate::*;

pub struct SequentialActionsPlugin {
    app_init: Box<dyn Fn(&mut App) + Send + Sync>,
}

impl SequentialActionsPlugin {
    pub fn new<F>(app_init: F) -> Self
    where
        F: Fn(&mut App) + Send + Sync + 'static,
    {
        Self {
            app_init: Box::new(app_init),
        }
    }
}

impl Default for SequentialActionsPlugin {
    fn default() -> Self {
        Self::new(|app| {
            app.add_stage_after(
                CoreStage::PostUpdate,
                "check_actions",
                SystemStage::parallel(),
            )
            .add_system_set_to_stage(
                "check_actions",
                SystemSet::new().with_system(check_actions),
                // .with_system(reset_action_status.after(check_action_status)),
            );
        })
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        (self.app_init)(app);
    }
}

fn check_actions(
    mut q: Query<
        (Entity, &CurrentAction, &mut ActionStatus),
        (Changed<ActionStatus>, With<ActionMarker>),
    >,
    mut commands: Commands,
) {
    for (entity, current_action, mut status) in q.iter_mut() {
        if let Some((action_type, _)) = &current_action.0 {
            let is_finished = match action_type {
                ActionType::Single(_) => status.finished_count == 1,
                ActionType::Many(actions) => status.finished_count == actions.len() as u32,
            };

            dbg!(status.finished_count);
            status.finished_count = 0;

            // dbg!(cfg.finished);
            // cfg.finished = 0;

            if is_finished {
                commands.actions(entity).finish();
            }
        }
    }
}

fn reset_action_status(mut q: Query<&mut ActionStatus, With<ActionMarker>>) {
    for mut status in q.iter_mut() {
        status.finished_count = 0;
    }
}
