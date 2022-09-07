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
            .add_system_to_stage("check_actions", check_action_status);
        })
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        (self.app_init)(app);
    }
}

fn check_action_status(
    mut q: Query<(Entity, &mut CurrentAction), With<ActionMarker>>,
    mut commands: Commands,
) {
    for (entity, mut current_action) in q.iter_mut() {
        if let Some((action_type, cfg)) = &mut current_action.0 {
            let is_finished = match action_type {
                ActionType::Single(_) => cfg.finished == 1,
                ActionType::Many(actions) => cfg.finished == actions.len() as u32, // TODO?
            };

            dbg!(cfg.finished);

            cfg.finished = 0;

            if is_finished {
                commands.actions(entity).next();
            }
        }
    }
}
