use bevy_app::{App, CoreStage, Plugin};

use crate::*;

/// The [`Plugin`] for `bevy-sequential-actions`.
/// This must be added to [`App`] in order for everything to work.
///
/// # Example
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
#[allow(clippy::needless_doctest_main)]
pub struct SequentialActionsPlugin;

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::PostUpdate,
            CHECK_ACTIONS_STAGE,
            SystemStage::single(check_actions),
        );
    }
}

pub(super) const CHECK_ACTIONS_STAGE: &str = "check_actions";

#[allow(clippy::type_complexity)]
pub(super) fn check_actions(
    mut q: Query<
        (Entity, &CurrentAction, &mut AgentState),
        (Changed<AgentState>, With<ActionMarker>),
    >,
    mut commands: Commands,
) {
    for (agent, current_action, mut finished_count) in q.iter_mut() {
        if let Some((action_type, _)) = &current_action.0 {
            let action_count = match action_type {
                ActionType::Single(_) => 1,
                ActionType::Multiple(actions) => actions.len() as u32,
            };

            if finished_count.total() == action_count {
                commands.add(move |world: &mut World| {
                    world.finish_action(agent);
                });
            }

            finished_count.finished_reset = 0;
        }
    }
}
