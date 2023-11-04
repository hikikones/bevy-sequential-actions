use bevy_ecs::query::ReadOnlyWorldQuery;

use crate::*;

/// The [`Plugin`] for this library that you can add to your [`App`].
///
/// This plugin adds the system [`check_actions`](Self::check_actions) to the [`Last`] schedule,
/// It also contains various static methods for modifying the action queue.
///
/// # Example
///
/// ```rust,no_run
/// # use bevy_ecs::prelude::*;
/// # use bevy_app::prelude::*;
/// # use bevy_sequential_actions::*;
/// # fn main() {
/// App::new()
///     .add_plugins(SequentialActionsPlugin)
///     .run();
/// # }
/// ```
pub struct SequentialActionsPlugin;

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, Self::check_actions::<()>);
    }
}

impl SequentialActionsPlugin {
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
    ///     .add_systems(Last, SequentialActionsPlugin::check_actions::<()>)
    ///     .run();
    /// # }
    /// ```
    pub fn check_actions<F: ReadOnlyWorldQuery>(
        action_q: Query<(Entity, &CurrentAction), F>,
        world: &World,
        mut commands: Commands,
    ) {
        action_q.for_each(|(agent, current_action)| {
            if let Some(action) = current_action.as_ref() {
                if action.is_finished(agent, world) {
                    commands.add(move |world: &mut World| {
                        Self::stop_current_action(agent, StopReason::Finished, world);
                        Self::start_next_action(agent, world);
                    });
                }
            }
        });
    }

    /// Adds a single [`action`](Action) to `agent` with specified `config`.
    pub fn add_action(
        agent: Entity,
        config: AddConfig,
        action: impl Into<BoxedAction>,
        world: &mut World,
    ) {
        let mut action = action.into();
        action.on_add(agent, world);

        let mut queue = world.get_mut::<ActionQueue>(agent).unwrap();
        match config.order {
            AddOrder::Back => queue.push_back(action),
            AddOrder::Front => queue.push_front(action),
        }

        if config.start && world.get::<CurrentAction>(agent).unwrap().is_none() {
            Self::start_next_action(agent, world);
        }
    }

    /// Adds a collection of actions to `agent` with specified `config`.
    pub fn add_actions<I>(agent: Entity, config: AddConfig, actions: I, world: &mut World)
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator,
    {
        match config.order {
            AddOrder::Back => {
                actions.into_iter().for_each(|mut action| {
                    action.on_add(agent, world);
                    world
                        .get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_back(action);
                });
            }
            AddOrder::Front => {
                actions.into_iter().rev().for_each(|mut action| {
                    action.on_add(agent, world);
                    world
                        .get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_front(action);
                });
            }
        }

        if config.start && world.get::<CurrentAction>(agent).unwrap().is_none() {
            Self::start_next_action(agent, world);
        }
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue for `agent`,
    /// but only if there is no current action.
    pub fn execute_actions(agent: Entity, world: &mut World) {
        if world.get::<CurrentAction>(agent).unwrap().is_none() {
            Self::start_next_action(agent, world);
        }
    }

    /// [`Stops`](Action::on_stop) the current [`action`](Action) for `agent` with specified `reason`.
    pub fn stop_current_action(agent: Entity, reason: StopReason, world: &mut World) {
        if let Some(mut action) = world.get_mut::<CurrentAction>(agent).unwrap().take() {
            action.on_stop(agent, world, reason);

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    action.on_remove(agent, world);
                    action.on_drop(agent, world, DropReason::Done);
                }
                StopReason::Paused => {
                    world
                        .get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_front(action);
                }
            }
        }
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue for `agent`.
    pub fn start_next_action(agent: Entity, world: &mut World) {
        if let Some(mut next_action) = world.get_mut::<ActionQueue>(agent).unwrap().pop_front() {
            if next_action.on_start(agent, world) {
                next_action.on_stop(agent, world, StopReason::Finished);
                next_action.on_remove(agent, world);
                next_action.on_drop(agent, world, DropReason::Done);
                Self::start_next_action(agent, world);
                return;
            }

            if let Some(mut current_action) = world.get_mut::<CurrentAction>(agent) {
                current_action.0 = Some(next_action);
            }
        }
    }

    /// Skips the next [`action`](Action) in the queue for `agent`.
    pub fn skip_next_action(agent: Entity, world: &mut World) {
        if let Some(mut action) = world.get_mut::<ActionQueue>(agent).unwrap().pop_front() {
            action.on_remove(agent, world);
            action.on_drop(agent, world, DropReason::Skipped);
        }
    }

    /// Clears the action queue for `agent`.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    pub fn clear_actions(agent: Entity, world: &mut World) {
        if let Some(mut action) = world.get_mut::<CurrentAction>(agent).unwrap().take() {
            action.on_stop(agent, world, StopReason::Canceled);
            action.on_remove(agent, world);
            action.on_drop(agent, world, DropReason::Cleared);
        }

        world
            .get_mut::<ActionQueue>(agent)
            .unwrap()
            .drain(..)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|mut action| {
                action.on_remove(agent, world);
                action.on_drop(agent, world, DropReason::Cleared);
            });
    }
}
