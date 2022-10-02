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
            SystemStage::single_threaded(),
        )
        .add_system_set_to_stage(
            CHECK_ACTIONS_STAGE,
            SystemSet::new()
                .with_system(count_finished_actions)
                .with_system(check_finished_actions.after(count_finished_actions)),
        );
    }
}

pub(super) const CHECK_ACTIONS_STAGE: &str = "check_actions";

pub(super) fn count_finished_actions(
    mut finished_q: Query<(&mut IsFinished, &ActionAgent), Changed<IsFinished>>,
    mut count_q: Query<&mut FinishedCount>,
) {
    for (mut finished, agent) in finished_q.iter_mut() {
        if finished.0 {
            count_q.get_mut(agent.0).unwrap().0 += 1;
        }

        finished.0 = false;
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn check_finished_actions(
    mut q: Query<
        (Entity, &CurrentAction, &mut FinishedCount),
        (Changed<FinishedCount>, With<ActionMarker>),
    >,
    mut commands: Commands,
) {
    for (agent, current_action, mut finished_count) in q.iter_mut() {
        if let Some((action_type, _)) = &current_action.0 {
            let action_count = match action_type {
                ActionType::Single(_) => 1,
                ActionType::Multiple(actions) => actions.len() as u32,
            };

            if finished_count.0 == action_count {
                commands.add(move |world: &mut World| {
                    world.finish_action(agent);
                });
            }

            finished_count.0 = 0;
        }
    }
}
