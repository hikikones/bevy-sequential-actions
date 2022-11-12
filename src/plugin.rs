use bevy_app::{App, CoreStage, Plugin};

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// This plugin adds the necessary systems for advancing the action queue for each `agent`.
/// By default, the systems will be added to [`CoreStage::Last`].
/// If you want to schedule the systems yourself, use [`get_systems`](Self::get_systems).
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
pub struct SequentialActionsPlugin;

impl SequentialActionsPlugin {
    /// Returns the systems used by this plugin.
    /// Useful if you want to schedule the systems yourself.
    ///
    /// ```rust,no_run
    /// use bevy::prelude::*;
    /// use bevy_sequential_actions::*;
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_plugins(DefaultPlugins)
    ///         .add_system_set_to_stage(CoreStage::Last, SequentialActionsPlugin::get_systems())
    ///         .run();
    /// }
    /// ```
    pub fn get_systems() -> SystemSet {
        SystemSet::new().with_system(check_actions)
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(CoreStage::Last, Self::get_systems());
    }
}

#[allow(clippy::type_complexity)]
fn check_actions(
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
