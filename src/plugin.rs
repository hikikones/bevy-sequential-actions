use std::marker::PhantomData;

use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::SystemConfigs;

use crate::*;

/// The [`Plugin`] for this library that must be added to [`App`] in order for everything to work.
///
/// This plugin adds the necessary systems for advancing the action queue for each `agent`.
/// By default, the systems will be added to [`CoreSet::Last`].
///
/// If you want to customize the scheduling, see [`new`](Self::new).
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
pub struct SequentialActionsPlugin<T: AgentMarker = DefaultAgentMarker> {
    builder: Box<dyn Fn(&mut App) + Send + Sync>,
    _marker: PhantomData<T>,
}

/// Trait alias for marker components used in [`SequentialActionsPlugin`].
pub trait AgentMarker: Default + Component {}

impl<T> AgentMarker for T where T: Default + Component {}

impl Default for SequentialActionsPlugin<DefaultAgentMarker> {
    fn default() -> Self {
        Self::new(|app: &mut App| {
            app.add_systems(Self::get_systems().in_base_set(CoreSet::Last));
        })
    }
}

impl<T: AgentMarker> SequentialActionsPlugin<T> {
    /// Creates a new [`Plugin`] with a closure for custom scheduling.
    /// To get the systems used by this plugin, use [`get_systems`](Self::get_systems).
    ///
    /// ```rust,no_run
    /// # use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
    /// # use bevy_app::prelude::*;
    /// use bevy_sequential_actions::*;
    ///
    /// fn main() {
    ///     App::new()
    ///         .add_schedule(CustomSchedule, Schedule::new())
    ///         .add_plugin(SequentialActionsPlugin::<CustomMarker>::new(
    ///             |app: &mut App| {
    ///                 app.add_systems(
    ///                     SequentialActionsPlugin::<CustomMarker>::get_systems()
    ///                         .in_schedule(CustomSchedule),
    ///                 );
    ///             },
    ///         ))
    ///         .add_startup_system(setup)
    ///         .add_system(run_custom_schedule)
    ///         .run();
    /// }
    ///
    /// #[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
    /// struct CustomSchedule;
    ///
    /// #[derive(Default, Component)]
    /// struct CustomMarker;
    ///
    /// fn run_custom_schedule(world: &mut World) {
    ///     world.run_schedule(CustomSchedule);
    /// }
    ///
    /// fn setup(mut commands: Commands) {
    ///     let agent = commands.spawn(ActionsBundle::<CustomMarker>::new()).id();
    ///     // ...
    /// }
    /// ```
    pub fn new(f: impl Fn(&mut App) + Send + Sync + 'static) -> Self {
        Self {
            builder: Box::new(f),
            _marker: PhantomData,
        }
    }

    /// Returns the systems used by this plugin.
    pub fn get_systems() -> SystemConfigs {
        (check_actions::<T>,).into_configs()
    }
}

impl<T: AgentMarker> Plugin for SequentialActionsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeferredActions>();
        (self.builder)(app);
    }
}

fn check_actions<T: AgentMarker>(
    action_q: Query<(Entity, &CurrentAction), With<T>>,
    world: &World,
    mut commands: Commands,
) {
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
