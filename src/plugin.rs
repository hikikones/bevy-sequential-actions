use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::{query::ReadOnlyWorldQuery, system::BoxedSystem};

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// This plugin adds a system that advances the action queue for each `agent`.
/// By default, the system is added to [`CoreSet::Last`].
/// For custom scheduling, see [`ActionHandler::check_actions`].
///
/// # Example
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_app::prelude::*;
/// # use bevy_sequential_actions::*;
/// # fn main() {
/// App::new()
///     .add_plugin(SequentialActionsPlugin)
///     .run();
/// # }
/// ```
pub struct SequentialActionsPlugin;

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(check_actions::<()>.in_base_set(CoreSet::Last));
    }
}

impl ActionHandler {
    /// The [`System`] used by [`SequentialActionsPlugin`].
    /// It is responsible for checking all agents for finished actions
    /// and advancing the action queue.
    ///
    /// The query filter `F` is used for filtering agents.
    /// Use the unit type `()` for no filtering.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bevy_ecs::prelude::*;
    /// # use bevy_app::prelude::*;
    /// # use bevy_sequential_actions::*;
    /// #
    /// # fn main() {
    /// App::new()
    ///     .add_system(
    ///         ActionHandler::check_actions::<()>().in_base_set(CoreSet::Last)
    ///     )
    ///     .run();
    /// # }
    /// ```
    pub fn check_actions<F: ReadOnlyWorldQuery + 'static>() -> BoxedSystem {
        Box::new(IntoSystem::into_system(check_actions::<F>))
    }
}

fn check_actions<F: ReadOnlyWorldQuery>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    mut commands: Commands,
) {
    action_q.for_each(|(agent, current_action)| {
        if let Some(action) = current_action.as_ref() {
            if action.is_finished(agent, world) {
                commands.add(move |world: &mut World| {
                    ActionHandler::stop_current(agent, StopReason::Finished, world);
                    ActionHandler::start_next(agent, world);
                });
            }
        }
    });
}
