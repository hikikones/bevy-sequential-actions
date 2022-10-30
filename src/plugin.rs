use bevy_app::{App, CoreStage, Plugin};
use bevy_ecs::schedule::StageLabelId;

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_sequential_actions::*;
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugin(SequentialActionsPlugin)
///         .run();
/// }
/// ```
#[derive(Default)]
pub struct SequentialActionsPlugin {
    stage_label_id: Option<StageLabelId>,
}

impl SequentialActionsPlugin {
    pub fn new(stage_label: impl StageLabel) -> Self {
        Self {
            stage_label_id: Some(stage_label.as_label()),
        }
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        if let Some(stage) = &self.stage_label_id {
            app.add_system_to_stage(stage.clone(), check_actions);
            return;
        }

        app.add_stage_after(
            CoreStage::PostUpdate,
            "CHECK_ACTIONS_STAGE",
            SystemStage::single(check_actions),
        );
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn check_actions(
    mut q: Query<(Entity, &CurrentAction, &mut ActionFinished), Changed<ActionFinished>>,
    mut commands: Commands,
) {
    for (agent, current_action, mut finished) in q.iter_mut() {
        if let Some((current_action, _)) = &current_action.0 {
            // TODO: Add debug warning when total > len.
            if finished.total() == current_action.len() {
                commands.add(move |world: &mut World| {
                    world.finish_action(agent);
                });
            }

            finished.reset_count = 0;
        }
    }
}
