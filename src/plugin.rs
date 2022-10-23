use bevy_app::{App, CoreStage, Plugin};

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
        (Entity, &CurrentAction, &mut ActionFinished),
        (Changed<ActionFinished>, With<ActionMarker>),
    >,
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
