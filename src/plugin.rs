use std::marker::PhantomData;

use bevy_app::{App, CoreSchedule, CoreSet, IntoSystemAppConfig, IntoSystemAppConfigs, Plugin};
use bevy_ecs::schedule::{ScheduleLabel, SystemConfigs};

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
pub struct SequentialActionsPlugin<T: Marker = DefaultMarker> {
    custom: Box<dyn Fn(&mut App) + Send + Sync>,
    _marker: PhantomData<T>,
}

pub trait Marker: Default + Clone + Copy + Component {}

impl<T> Marker for T where T: Default + Clone + Copy + Component {}

#[derive(Default, Clone, Copy, Component)]
pub struct DefaultMarker;

// TODO: Rework custom scheduling.
// Cannot use get_systems() currently as the DeferredActions resource is now required.

impl Default for SequentialActionsPlugin<DefaultMarker> {
    fn default() -> Self {
        Self::custom(|app: &mut App| {
            app.add_systems(Self::get_systems().in_base_set(CoreSet::Last));
        })
    }
}

// impl<T: Marker> Default for SequentialActionsPlugin<T> {
//     fn default() -> Self {
//         Self::custom(|app: &mut App| {
//             app.add_systems(Self::get_systems().in_base_set(CoreSet::Last));
//         })
//     }
// }

impl<T: Marker> SequentialActionsPlugin<T> {
    pub fn custom(f: impl Fn(&mut App) + Send + Sync + 'static) -> Self {
        Self {
            custom: Box::new(f),
            _marker: PhantomData,
        }
    }

    pub fn get_systems() -> SystemConfigs {
        (check_actions::<T>,).into_configs()
    }
}

impl<T: Marker> Plugin for SequentialActionsPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeferredActions>();
        (self.custom)(app);
    }
}

fn check_actions<T: Marker>(
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
