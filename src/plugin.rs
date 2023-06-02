use std::{cmp::Ordering, marker::PhantomData};

use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::{
    query::ReadOnlyWorldQuery,
    system::{BoxedSystem, SystemState},
};

use crate::*;

type SortFn = fn(Entity, Entity, &World) -> Ordering;

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
    sort_fn: Option<SortFn>,
    _filter: PhantomData<F>,
}

impl Default for SequentialActionsPlugin {
    fn default() -> Self {
        Self::new(
            QueueAdvancement::Normal,
            |app, system| {
                app.add_system(system.in_base_set(CoreSet::Last));
            },
            None,
        )
    }
}

impl<F: ReadOnlyWorldQuery> SequentialActionsPlugin<F> {
    /// Creates a new [`Plugin`] with specified [`QueueAdvancement`].
    /// The closure `init_fn` provides the system used by this plugin.
    /// Add this system to your app with any constraints you may have.
    /// The `sort_fn` argument is an optional function
    /// for sorting the query that checks all agents for finished actions.
    ///
    /// The query filter `F` is used for filtering agents
    /// and is applied to the system provided by the closure.
    /// Use the unit type `()` for no filtering.
    ///
    /// Note that `sort_fn` can only be defined once per unique type `F`.
    /// Adding the plugin multiple times with the same type `F`
    /// and different `sort_fn` will end up with only one `sort_fn`,
    /// as they will overwrite each other.
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
    ///         },
    ///         None
    ///     ))
    ///     .run();
    /// # }
    /// ```
    pub fn new(
        system_kind: QueueAdvancement,
        init_fn: impl Fn(&mut App, BoxedSystem) + Send + Sync + 'static,
        sort_fn: Option<SortFn>,
    ) -> Self {
        Self {
            system_kind,
            app_init: Box::new(init_fn),
            sort_fn,
            _filter: PhantomData,
        }
    }
}

impl<F: ReadOnlyWorldQuery + Send + Sync + 'static> Plugin for SequentialActionsPlugin<F> {
    fn build(&self, app: &mut App) {
        match self.system_kind {
            QueueAdvancement::Normal => {
                if let Some(sort_fn) = self.sort_fn {
                    app.insert_resource(AgentSortRes::<F> {
                        sort_fn,
                        _marker: PhantomData,
                    });
                    (self.app_init)(
                        app,
                        Box::new(IntoSystem::into_system(check_actions_normal_sorted::<F>)),
                    );
                } else {
                    (self.app_init)(
                        app,
                        Box::new(IntoSystem::into_system(check_actions_normal::<F>)),
                    );
                }
            }
            QueueAdvancement::Parallel => {
                if let Some(sort_fn) = self.sort_fn {
                    app.insert_resource(AgentSortRes::<F> {
                        sort_fn,
                        _marker: PhantomData,
                    });
                    (self.app_init)(
                        app,
                        Box::new(IntoSystem::into_system(check_actions_parallel_sorted::<F>)),
                    );
                } else {
                    (self.app_init)(
                        app,
                        Box::new(IntoSystem::into_system(check_actions_parallel::<F>)),
                    );
                }
            }
            QueueAdvancement::Exclusive => {
                app.init_resource::<AgentQueryRes<F>>();

                if let Some(sort_fn) = self.sort_fn {
                    app.insert_resource(AgentSortRes::<F> {
                        sort_fn,
                        _marker: PhantomData,
                    });
                    (self.app_init)(
                        app,
                        Box::new(IntoSystem::into_system(check_actions_exclusive_sorted::<F>)),
                    );
                } else {
                    (self.app_init)(
                        app,
                        Box::new(IntoSystem::into_system(check_actions_exclusive::<F>)),
                    );
                }
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

#[derive(Resource)]
struct AgentQueryRes<F: ReadOnlyWorldQuery + 'static>(
    SystemState<Query<'static, 'static, (Entity, &'static CurrentAction), F>>,
);

impl<F: ReadOnlyWorldQuery> FromWorld for AgentQueryRes<F> {
    fn from_world(world: &mut World) -> Self {
        Self(SystemState::new(world))
    }
}

#[derive(Resource)]
struct AgentSortRes<F: ReadOnlyWorldQuery> {
    sort_fn: SortFn,
    _marker: PhantomData<F>,
}

fn check_actions_normal<F>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    mut commands: Commands,
) where
    F: ReadOnlyWorldQuery,
{
    action_q.for_each(|(agent, current_action)| {
        if let Some(action) = current_action.as_ref() {
            if action.is_finished(agent, world).0 {
                commands.add(move |world: &mut World| {
                    world.stop_current_action(agent, StopReason::Finished);
                });
            }
        }
    });
}

fn check_actions_normal_sorted<F>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    sort_fn: Res<AgentSortRes<F>>,
    mut commands: Commands,
) where
    F: ReadOnlyWorldQuery + Send + Sync + 'static,
{
    let mut finished_agents = action_q
        .iter()
        .filter(|&(agent, current_action)| {
            if let Some(action) = current_action.as_ref() {
                return action.is_finished(agent, world).into();
            }
            false
        })
        .map(|(agent, _)| agent)
        .collect::<Vec<_>>();
    finished_agents.sort_unstable_by(|e1, e2| (sort_fn.sort_fn)(*e1, *e2, world));

    finished_agents.into_iter().for_each(|agent| {
        commands.add(move |world: &mut World| {
            world.stop_current_action(agent, StopReason::Finished);
        });
    });
}

fn check_actions_parallel<F>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    par_commands: ParallelCommands,
) where
    F: ReadOnlyWorldQuery,
{
    action_q.for_each(|(agent, current_action)| {
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

fn check_actions_parallel_sorted<F>(
    action_q: Query<(Entity, &CurrentAction), F>,
    world: &World,
    sort_fn: Res<AgentSortRes<F>>,
    par_commands: ParallelCommands,
) where
    F: ReadOnlyWorldQuery + Send + Sync + 'static,
{
    let mut finished_agents = action_q
        .iter()
        .filter(|&(agent, current_action)| {
            if let Some(action) = current_action.as_ref() {
                return action.is_finished(agent, world).into();
            }
            false
        })
        .map(|(agent, _)| agent)
        .collect::<Vec<_>>();
    finished_agents.sort_unstable_by(|e1, e2| (sort_fn.sort_fn)(*e1, *e2, world));

    finished_agents.into_iter().for_each(|agent| {
        par_commands.command_scope(|mut commands: Commands| {
            commands.add(move |world: &mut World| {
                world.stop_current_action(agent, StopReason::Finished);
            });
        });
    });
}

fn check_actions_exclusive<F>(world: &mut World)
where
    F: ReadOnlyWorldQuery + 'static,
{
    world.resource_scope(|world, mut system_state: Mut<AgentQueryRes<F>>| {
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

        finished_agents.into_iter().for_each(|agent| {
            world.stop_current_action(agent, StopReason::Finished);
        });
    });
}

fn check_actions_exclusive_sorted<F>(world: &mut World)
where
    F: ReadOnlyWorldQuery + Send + Sync + 'static,
{
    world.resource_scope(|world, mut system_state: Mut<AgentQueryRes<F>>| {
        let agent_q = system_state.0.get(world);

        let mut finished_agents = agent_q
            .iter()
            .filter(|&(agent, current_action)| {
                if let Some(action) = current_action.as_ref() {
                    return action.is_finished(agent, world).into();
                }
                false
            })
            .map(|(agent, _)| agent)
            .collect::<Vec<_>>();
        let sort_fn = world.resource::<AgentSortRes<F>>().sort_fn;
        finished_agents.sort_unstable_by(|e1, e2| sort_fn(*e1, *e2, world));

        finished_agents.into_iter().for_each(|agent| {
            world.stop_current_action(agent, StopReason::Finished);
        });
    });
}
