use std::marker::PhantomData;

use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::{
    query::ReadOnlyWorldQuery,
    system::{BoxedSystem, SystemState},
};

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// In short, this plugin adds a system that advances the action queue for each `agent`.
/// By default, the system is added to [`CoreSet::Last`].
/// For more control over scheduling, see [`new`](Self::new).
///
/// # Example
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_app::prelude::*;
/// # use bevy_sequential_actions::*;
/// # fn main() {
/// App::new()
///     .add_plugin(SequentialActionsPlugin::default())
///     .run();
/// # }
/// ```
#[allow(clippy::type_complexity)]
pub struct SequentialActionsPlugin<F: ReadOnlyWorldQuery = ()> {
    system_kind: QueueAdvancement,
    app_init: Box<dyn Fn(&mut App, BoxedSystem) + Send + Sync>,
    _filter: PhantomData<F>,
}

impl Default for SequentialActionsPlugin {
    fn default() -> Self {
        Self::new(QueueAdvancement::Normal, |app, system| {
            app.add_system(system.in_base_set(CoreSet::Last));
        })
    }
}

impl<F: ReadOnlyWorldQuery> SequentialActionsPlugin<F> {
    /// Creates a new [`Plugin`] with specified [`QueueAdvancement`].
    /// The closure `f` provides the system used by this plugin.
    /// Add this system to your app with any constraints you may have.
    ///
    /// The query filter `F` is used for filtering agents
    /// and is applied to the system provided by the closure.
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
    ///     .add_plugin(SequentialActionsPlugin::<()>::new(
    ///         QueueAdvancement::Normal,
    ///         |app, system| {
    ///             app.add_system(system.in_base_set(CoreSet::Last));
    ///         }
    ///     ))
    ///     .run();
    /// # }
    /// ```
    pub fn new(
        system_kind: QueueAdvancement,
        f: impl Fn(&mut App, BoxedSystem) + Send + Sync + 'static,
    ) -> Self {
        Self {
            system_kind,
            app_init: Box::new(f),
            _filter: PhantomData,
        }
    }
}

impl<F: ReadOnlyWorldQuery + Send + Sync + 'static> Plugin for SequentialActionsPlugin<F> {
    fn build(&self, app: &mut App) {
        match self.system_kind {
            QueueAdvancement::Normal => {
                (self.app_init)(
                    app,
                    Box::new(IntoSystem::into_system(check_actions_normal::<F>)),
                );
            }
            QueueAdvancement::Parallel => {
                (self.app_init)(
                    app,
                    Box::new(IntoSystem::into_system(check_actions_parallel::<F>)),
                );
            }
            QueueAdvancement::Exclusive => {
                app.init_resource::<CachedAgentQuery<F>>();
                (self.app_init)(
                    app,
                    Box::new(IntoSystem::into_system(check_actions_exclusive::<F>)),
                );
            }
        }
    }
}

/// The kinds of systems available for advancing the action queue.
/// For use in [`SequentialActionsPlugin::new`].
///
/// `Normal` uses [`Commands`], `Parallel` uses [`ParallelCommands`],
/// and `Exclusive` uses [`World`] directly (no commands).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum QueueAdvancement {
    /// Advances the action queue using [`Commands`].
    #[default]
    Normal,
    /// Advances the action queue using [`ParallelCommands`].
    /// Useful when you have lots of agents such as tens of thousands or more.
    Parallel,
    /// Advances the action queue using [`World`] in an exclusive system.
    Exclusive,
}

fn check_actions_normal<F: ReadOnlyWorldQuery>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    mut commands: Commands,
) {
    for (agent, current_action) in action_q.iter() {
        if let Some(action) = current_action.as_ref() {
            if action.is_finished(agent, world).0 {
                commands.add(move |world: &mut World| {
                    world.stop_current_action(agent, StopReason::Finished);
                });
            }
        }
    }
}

fn check_actions_parallel<F: ReadOnlyWorldQuery>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    par_commands: ParallelCommands,
) {
    action_q.par_iter().for_each(|(agent, current_action)| {
        if let Some(action) = current_action.as_ref() {
            if action.is_finished(agent, world).0 {
                par_commands.command_scope(|mut commands: Commands| {
                    commands.add(move |world: &mut World| {
                        world.stop_current_action(agent, StopReason::Finished);
                    });
                });
            }
        }
    });
}

#[derive(Resource)]
struct CachedAgentQuery<F: ReadOnlyWorldQuery + 'static>(
    SystemState<Query<'static, 'static, (Entity, &'static CurrentAction), F>>,
);

impl<F: ReadOnlyWorldQuery> FromWorld for CachedAgentQuery<F> {
    fn from_world(world: &mut World) -> Self {
        Self(SystemState::new(world))
    }
}

fn check_actions_exclusive<F: ReadOnlyWorldQuery + 'static>(world: &mut World) {
    world.resource_scope(|world, mut system_state: Mut<CachedAgentQuery<F>>| {
        let agent_q = system_state.0.get(world);

        let finished_agents = agent_q
            .iter()
            .filter(|&(agent, current_action)| {
                if let Some(action) = current_action.as_ref() {
                    return action.is_finished(agent, world).into();
                }
                false
            })
            .map(|(agent, _)| agent)
            .collect::<Vec<_>>();

        for agent in finished_agents {
            world.stop_current_action(agent, StopReason::Finished);
        }
    });
}
