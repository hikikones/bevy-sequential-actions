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
///         .add_plugin(SequentialActionsPlugin::default())
///         .run();
/// }
/// ```
pub struct SequentialActionsPlugin {
    stage_label_id: StageLabelId,
}

impl SequentialActionsPlugin {
    /// Creates a new plugin with specified [`StageLabel`].
    /// A single [`System`] will be added to this stage
    /// that checks for finished actions.
    /// By default, the [`CoreStage::Last`] is used.
    pub fn new(stage_label: impl StageLabel) -> Self {
        Self {
            stage_label_id: stage_label.as_label(),
        }
    }
}

impl Default for SequentialActionsPlugin {
    fn default() -> Self {
        Self::new(CoreStage::Last)
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(self.stage_label_id, check_actions);
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
