use std::marker::PhantomData;

use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::system::{BoxedSystem, SystemState};

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// In short, this plugin adds a system that advances the action queue for each `agent`.
/// By default, the system is added to [`CoreSet::Last`].
/// For more control over scheduling, see [`new`](Self::new).
///
/// The generic marker type `T` is used for filtering agents,
/// allowing you to add the plugin multiple times for each type `T`.
/// By default, the [`DefaultAgentMarker`] is used.
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
pub struct SequentialActionsPlugin<T: AgentMarker> {
    system_kind: QueueAdvancement,
    app_init: Box<dyn Fn(&mut App, BoxedSystem) + Send + Sync>,
    _marker: PhantomData<T>,
}

impl Default for SequentialActionsPlugin<DefaultAgentMarker> {
    fn default() -> Self {
        Self::new(QueueAdvancement::Normal, |app, system| {
            app.add_system(system.in_base_set(CoreSet::Last));
        })
    }
}

impl<T: AgentMarker> SequentialActionsPlugin<T> {
    /// Creates a new [`Plugin`] with specified [`QueueAdvancement`].
    /// The closure `f` provides the system used by this plugin
    /// for advancing the action queue for each `agent`.
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
    ///     .add_plugin(SequentialActionsPlugin::<DefaultAgentMarker>::new(
    ///         QueueAdvancement::Normal,
    ///         |app, system| {
    ///             app.add_system(system.in_base_set(CoreSet::Last));
    ///         }
    ///     ))
    ///     .run();
    /// # }
    /// ```
    pub fn new<F>(system_kind: QueueAdvancement, f: F) -> Self
    where
        F: Fn(&mut App, BoxedSystem) + Send + Sync + 'static,
    {
        Self {
            system_kind,
            app_init: Box::new(f),
            _marker: PhantomData,
        }
    }
}

impl<T: AgentMarker> Plugin for SequentialActionsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<CachedAgentQuery<T>>();

        (self.app_init)(
            app,
            match self.system_kind {
                QueueAdvancement::Normal => {
                    Box::new(IntoSystem::into_system(check_actions_normal::<T>))
                }
                QueueAdvancement::Parallel => {
                    Box::new(IntoSystem::into_system(check_actions_parallel::<T>))
                }
                QueueAdvancement::Exclusive => {
                    Box::new(IntoSystem::into_system(check_actions_exclusive::<T>))
                }
            },
        );
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

fn check_actions_normal<T: AgentMarker>(
    action_q: Query<(Entity, &CurrentAction), With<T>>,
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

fn check_actions_parallel<T: AgentMarker>(
    action_q: Query<(Entity, &CurrentAction), With<T>>,
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
struct CachedAgentQuery<T: AgentMarker>(
    SystemState<Query<'static, 'static, (Entity, &'static CurrentAction), With<T>>>,
);

impl<T: AgentMarker> FromWorld for CachedAgentQuery<T> {
    fn from_world(world: &mut World) -> Self {
        Self(SystemState::new(world))
    }
}

fn check_actions_exclusive<T: AgentMarker>(world: &mut World) {
    world.resource_scope(|world, mut system_state: Mut<CachedAgentQuery<T>>| {
        let agent_q = system_state.0.get(world);

        let finished_agents = agent_q
            .iter()
            .filter(|(agent, current_action)| {
                if let Some(action) = current_action.as_ref() {
                    return action.is_finished(*agent, world).into();
                }
                false
            })
            .map(|(e, _)| e)
            .collect::<Vec<_>>();

        for agent in finished_agents {
            world.stop_current_action(agent, StopReason::Finished);
        }
    });
}
