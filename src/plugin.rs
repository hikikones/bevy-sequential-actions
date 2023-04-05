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
/// # use bevy_ecs::prelude::*;
/// # use bevy_app::prelude::*;
/// use bevy_sequential_actions::*;
///
/// fn main() {
///     App::new()
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
    /// use bevy_ecs::prelude::*;
    /// use bevy_app::prelude::*;
    /// use bevy_sequential_actions::*;
    ///
    /// fn main() {
    ///     App::new()
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

fn check_actions(action_q: Query<(Entity, &CurrentAction)>, world: &World, mut commands: Commands) {
    for (agent, current_action) in action_q.iter() {
        if let Some(action) = &current_action.0 {
            if action.is_finished(agent, world) {
                commands.add(move |world: &mut World| {
                    world.stop_current_action(agent, StopReason::Finished);
                });
            }
        }
    }
}
