use std::marker::PhantomData;

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
///         .add_plugin(SequentialActionsPlugin::default())
///         .run();
/// }
/// ```
pub struct SequentialActionsPlugin<M: AgentMarker = DefaultAgentMarker>(PhantomData<M>);

impl Default for SequentialActionsPlugin<DefaultAgentMarker> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: AgentMarker> SequentialActionsPlugin<T> {
    /// Returns the systems used by this plugin for advancing the action queue for each `agent`.
    /// Finished actions are queued using [`Commands`].
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
    ///     .add_systems(
    ///         SequentialActionsPlugin::<DefaultAgentMarker>::get_systems()
    ///             .in_base_set(CoreSet::Last)
    ///     )
    ///     .run();
    /// # }
    /// ```
    pub fn get_systems() -> SystemConfigs {
        (check_actions::<T>,).into_configs()
    }

    /// Returns the systems used by this plugin for advancing the action queue for each `agent`.
    /// Finished actions are queued using [`ParallelCommands`].
    /// Use this when you have lots of agents such as tens of thousands or more.
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
    ///     .add_systems(
    ///         SequentialActionsPlugin::<DefaultAgentMarker>::get_parallel_systems()
    ///             .in_base_set(CoreSet::Last)
    ///     )
    ///     .run();
    /// # }
    /// ```
    pub fn get_parallel_systems() -> SystemConfigs {
        (check_actions_par::<T>,).into_configs()
    }
}

impl<T: AgentMarker> Plugin for SequentialActionsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(Self::get_systems().in_base_set(CoreSet::Last));
    }
}

fn check_actions<T: AgentMarker>(
    action_q: Query<(Entity, &CurrentAction), With<T>>,
    world: &World,
    mut commands: Commands,
) {
    for (agent, current_action) in action_q.iter() {
        if let Some(action) = current_action.as_ref() {
            if action.is_finished(agent, world) {
                commands.add(move |world: &mut World| {
                    world.stop_current_action(agent, StopReason::Finished);
                });
            }
        }
    }
}

fn check_actions_par<T: AgentMarker>(
    action_q: Query<(Entity, &CurrentAction), With<T>>,
    world: &World,
    par_commands: ParallelCommands,
) {
    action_q.par_iter().for_each(|(agent, current_action)| {
        if let Some(action) = current_action.as_ref() {
            if action.is_finished(agent, world) {
                par_commands.command_scope(|mut commands: Commands| {
                    commands.add(move |world: &mut World| {
                        world.stop_current_action(agent, StopReason::Finished);
                    });
                });
            }
        }
    });
}
