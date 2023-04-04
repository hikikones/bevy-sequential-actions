use std::cmp::Ordering;

use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::SystemConfigs;

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// This plugin adds the necessary systems for advancing the action queue for each `agent`.
/// By default, the systems will be added to [`CoreSet::Last`].
/// If you want to schedule the systems yourself, see [`get_systems`](Self::get_systems).
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

// TODO: Rework custom scheduling.
// Cannot use get_systems() currently as the DeferredActions resource is now required.

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
    ///         .add_systems(SequentialActionsPlugin::get_systems().in_base_set(CoreSet::Last))
    ///         .run();
    /// }
    /// ```
    pub fn get_systems() -> SystemConfigs {
        (check_actions,).into_configs()
    }
}

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeferredActions>()
            .add_systems(Self::get_systems().in_base_set(CoreSet::Last));
    }
}

fn check_actions(
    mut action_q: Query<(Entity, &CurrentAction, &mut ActionFinished), Changed<ActionFinished>>,
    mut commands: Commands,
) {
    for (agent, current_action, mut finished) in action_q.iter_mut() {
        if let Some((current_action, _)) = &current_action.0 {
            let finished_count = finished.total();
            let active_count = current_action.len();

            match finished_count.cmp(&active_count) {
                Ordering::Less => {
                    finished.reset_count = 0;
                }
                Ordering::Equal => {
                    commands.add(move |world: &mut World| {
                        world.finish_action(agent);
                    });
                }
                Ordering::Greater => {
                    panic!(
                        "Finished actions exceeds active. \
                        Entity {agent:?} has {active_count} active action(s), \
                        but a total of {finished_count} action(s) have been confirmed finished."
                    );
                }
            }
        }
    }
}
