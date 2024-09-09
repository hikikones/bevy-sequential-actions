use super::*;

/// The [`Plugin`] for this library that you can add to your [`App`].
///
/// This plugin adds the system [`check_actions`](Self::check_actions) to the [`Last`] schedule.
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
    pub fn check_actions<F: QueryFilter>(
        action_q: Query<(Entity, &CurrentAction), F>,
        world: &World,
        mut commands: Commands,
    ) {
        action_q.iter().for_each(|(agent, current_action)| {
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
        debug!("Adding action {action:?} for agent {agent} with {config:?}.");
        action.on_add(agent, world);

        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            error!("Cannot enqueue action {action:?} to non-existent agent {agent}. \
            Action is therefore dropped immediately.");
            action.on_remove(None, world);
            action.on_drop(None, world, DropReason::Skipped);
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            error!("Cannot enqueue action {action:?} to agent {agent} due to missing component {}. \
            Action is therefore dropped immediately.", type_name::<ActionQueue>());
            action.on_remove(agent.into(), world);
            action.on_drop(agent.into(), world, DropReason::Skipped);
            return;
        };

        match config.order {
            AddOrder::Back => action_queue.push_back(action),
            AddOrder::Front => action_queue.push_front(action),
        }

        if config.start {
            let Some(current_action) = agent_ref.get::<CurrentAction>() else {
                error!("Could not start next action for agent {agent} due to missing component {}.", type_name::<CurrentAction>());
                return;
            };

            if current_action.is_none() {
                Self::start_next_action(agent, world);
            }
        }
    }

    /// Adds a collection of actions to `agent` with specified `config`.
    pub fn add_actions<I>(agent: Entity, config: AddConfig, actions: I, world: &mut World)
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator + ExactSizeIterator + Debug,
    {
        let actions = actions.into_iter();
        debug!("Adding actions {actions:?} for agent {agent} with {config:?}.");
        let len = actions.len();
        
        if len == 0 {
            return;
        }

        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            error!("Cannot add actions {actions:?} to non-existent agent {agent}.");
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            error!("Cannot add actions {actions:?} to agent {agent} due to missing component {}.", type_name::<ActionQueue>());
            return;
        };

        action_queue.reserve(len);

        match config.order {
            AddOrder::Back => {
                actions.for_each(|mut action| {
                    action.on_add(agent, world);

                    let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                        error!("Cannot enqueue action {action:?} to non-existent agent {agent}. \
                        Action is therefore dropped immediately.");
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Skipped);
                        return;
                    };
            
                    let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                        error!("Cannot enqueue action {action:?} to agent {agent} due to missing component {}. \
                        Action is therefore dropped immediately.", type_name::<ActionQueue>());
                        action.on_remove(agent.into(), world);
                        action.on_drop(agent.into(), world, DropReason::Skipped);
                        return;
                    };

                    action_queue.push_back(action);
                });
            }
            AddOrder::Front => {
                actions.rev().for_each(|mut action| {
                    action.on_add(agent, world);

                    let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                        error!("Cannot enqueue action {action:?} to non-existent agent {agent}. \
                        Action is therefore dropped immediately.");
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Skipped);
                        return;
                    };
            
                    let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                        error!("Cannot enqueue action {action:?} to agent {agent} due to missing component {}. \
                        Action is therefore dropped immediately.", type_name::<ActionQueue>());
                        action.on_remove(agent.into(), world);
                        action.on_drop(agent.into(), world, DropReason::Skipped);
                        return;
                    };

                    action_queue.push_front(action);
                });
            }
        }

        if config.start {
            let Some(current_action) = world.get::<CurrentAction>(agent) else {
                error!("Could not start next action for agent {agent} due to missing component {}.", type_name::<CurrentAction>());
                return;
            };

            if current_action.is_none() {
                Self::start_next_action(agent, world);
            }
        }
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue for `agent`,
    /// but only if there is no current action.
    pub fn execute_actions(agent: Entity, world: &mut World) {
        let Some(agent_ref) = world.get_entity(agent) else {
            error!("Cannot execute actions for non-existent agent {agent}.");
            return;
        };
        
        let Some(current_action) = agent_ref.get::<CurrentAction>() else {
            error!("Cannot execute actions for agent {agent} due to missing component {}.", type_name::<CurrentAction>());
            return;
        };
        
        if current_action.is_none() {
            debug!("Executing actions for agent {agent}.");
            Self::start_next_action(agent, world);
        }
    }

    /// [`Stops`](Action::on_stop) the current [`action`](Action) for `agent` with specified `reason`.
    pub fn stop_current_action(agent: Entity, reason: StopReason, world: &mut World) {
        let Some(mut current_action) = world.get_mut::<CurrentAction>(agent) else {
            error!("Cannot stop current action for agent {agent} with reason {reason:?} due to missing component {}.", type_name::<CurrentAction>());
            return;
        };

        if let Some(mut action) = current_action.take() {
            debug!("Stopping current action {action:?} for agent {agent} with reason {reason:?}.");
            action.on_stop(agent.into(), world, reason);

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    action.on_remove(agent.into(), world);
                    action.on_drop(agent.into(), world, DropReason::Done);
                }
                StopReason::Paused => {
                    let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                        error!("Cannot enqueue paused action {action:?} to non-existent agent {agent}. \
                        Action is therefore dropped immediately.");
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Skipped);
                        return;
                    };

                    let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                        error!("Cannot enqueue paused action {action:?} to agent {agent} due to missing component {}. \
                        Action is therefore dropped immediately.", type_name::<ActionQueue>());
                        action.on_remove(agent.into(), world);
                        action.on_drop(agent.into(), world, DropReason::Skipped);
                        return;
                    };

                    action_queue
                    .push_front(action);
                }
            }
        }
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue for `agent`.
    pub fn start_next_action(agent: Entity, world: &mut World) {
        loop {
            let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                error!("Cannot start next action for non-existent agent {agent}.");
                break;
            };

            let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                error!("Cannot start next action for agent {agent} due to missing component {}.", type_name::<ActionQueue>());
                break;
            };

            let Some(mut action) = action_queue.pop_front() else {
                break;
            };

            debug!("Starting action {action:?} for agent {agent}.");

            if !action.on_start(agent, world) {
                match world.get_mut::<CurrentAction>(agent) {
                    Some(mut current_action) => {
                        current_action.0 = Some(action)
                    },
                    None => {
                        action.on_stop(None, world, StopReason::Canceled);
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Done);
                    },
                }
                break;
            };

            let agent = world.get_entity(agent).map(|_| agent);
            action.on_stop(agent, world, StopReason::Finished);
            action.on_remove(agent, world);
            action.on_drop(agent, world, DropReason::Done);

            if agent.is_none() {
                break;
            }
        }
    }

    /// Skips the next [`action`](Action) in the queue for `agent`.
    pub fn skip_next_action(agent: Entity, world: &mut World) {
        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            error!("Cannot skip next action for non-existent agent {agent}.");
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            error!("Cannot skip next action for agent {agent} due to missing component {}.", type_name::<ActionQueue>());
            return;
        };

        if let Some(mut action) = action_queue.pop_front() {
            debug!("Skipping action {action:?} for agent {agent}.");
            action.on_remove(agent.into(), world);
            action.on_drop(agent.into(), world, DropReason::Skipped);
        }
    }

    /// Clears the action queue for `agent`.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    pub fn clear_actions(agent: Entity, world: &mut World) {
        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            error!("Cannot clear current action for non-existent agent {agent}.");
            return;
        };

        let Some(mut current_action) = agent_ref.get_mut::<CurrentAction>() else {
            error!("Cannot clear current action for agent {agent} due to missing component {}.", type_name::<CurrentAction>());
            return;
        };

        if let Some(mut action) = current_action.take() {
            debug!("Clearing current action {action:?} for agent {agent}.");
            action.on_stop(agent.into(), world, StopReason::Canceled);
            action.on_remove(agent.into(), world);
            action.on_drop(agent.into(), world, DropReason::Cleared);
        }

        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            error!("Cannot clear action queue for non-existent agent {agent}.");
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            error!("Cannot clear action queue for agent {agent} due to missing component {}.", type_name::<ActionQueue>());
            return;
        };

        if action_queue.is_empty() {
            return;
        }

        debug!("Clearing {action_queue:?} for {agent}.");

        action_queue
            .drain(..)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|mut action| {
                action.on_remove(agent.into(), world);
                action.on_drop(agent.into(), world, DropReason::Cleared);
            });
    }
}
