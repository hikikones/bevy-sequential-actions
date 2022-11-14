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
        SystemSet::new()
            .with_system(check_actions)
            .with_system(reset_count.after(check_actions))
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(CoreStage::Last, Self::get_systems());
    }
}

fn check_actions(
    action_q: Query<(Entity, &CurrentAction, &ActionFinished), Changed<ActionFinished>>,
    mut commands: Commands,
) {
    for (agent, current_action, finished) in action_q.iter() {
        if let Some((current_action, _)) = &current_action.0 {
            let finished_count = finished.total();
            let action_count = current_action.len();

            if finished_count == action_count {
                commands.add(move |world: &mut World| {
                    world.finish_action(agent);
                });
            } else if finished_count > action_count {
                panic!(
                    "Agent {agent:?} has {action_count} active action(s), \
                    but a total of {finished_count} action(s) have been confirmed finished."
                );
            }
        }
    }
}

fn reset_count(mut finished_q: Query<&mut ActionFinished, Changed<ActionFinished>>) {
    for mut finished in finished_q.iter_mut() {
        finished.bypass_change_detection().reset_count = 0;
    }
}
