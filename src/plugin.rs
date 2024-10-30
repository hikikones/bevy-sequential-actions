use super::*;

/// The [`Plugin`] for this library that you can add to your [`App`].
///
/// This plugin adds the [`check_actions`](Self::check_actions) system to the [`Last`] schedule
/// for action queue advancement, and also two [`hooks`](bevy_ecs::component::ComponentHooks)
/// for cleaning up actions from despawned agents.
///
/// Finally, it also contains various static methods for modifying the action queue.
pub struct SequentialActionsPlugin;

impl Plugin for SequentialActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, Self::check_actions::<()>);
        app.world_mut()
            .register_component_hooks::<CurrentAction>()
            .on_remove(CurrentAction::on_remove_hook);
        app.world_mut()
            .register_component_hooks::<ActionQueue>()
            .on_remove(ActionQueue::on_remove_hook);
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
        action_q
            .iter()
            .filter_map(|(agent, current_action)| {
                current_action
                    .as_ref()
                    .and_then(|action| action.is_finished(agent, world).then_some(agent))
            })
            .for_each(|agent| {
                commands.add(move |world: &mut World| {
                    Self::stop_current_action(agent, StopReason::Finished, world);
                    Self::start_next_action(agent, world);
                });
            });
    }

    /// Adds a single [`action`](Action) to `agent` with specified `config`.
    pub fn add_action(
        agent: Entity,
        config: AddConfig,
        mut action: BoxedAction,
        world: &mut World,
    ) {
        if world.get_entity(agent).is_none() {
            warn!("Cannot add action {action:?} to non-existent agent {agent}.");
            return;
        }

        debug!("Adding action {action:?} for agent {agent} with {config:?}.");
        action.on_add(agent, world);

        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            warn!(
                "Cannot enqueue action {action:?} to non-existent agent {agent}. \
                Action is therefore dropped immediately."
            );
            action.on_remove(None, world);
            action.on_drop(None, world, DropReason::Skipped);
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            warn!(
                "Cannot enqueue action {action:?} to agent {agent} due to missing component {}. \
                Action is therefore dropped immediately.",
                std::any::type_name::<ActionQueue>()
            );
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
                warn!(
                    "Could not start next action for agent {agent} due to missing component {}.",
                    std::any::type_name::<CurrentAction>()
                );
                return;
            };

            if current_action.is_none() {
                Self::start_next_action(agent, world);
            }
        }
    }

    /// Adds a collection of actions to `agent` with specified `config`.
    /// An empty collection does nothing.
    pub fn add_actions<I>(agent: Entity, config: AddConfig, actions: I, world: &mut World)
    where
        I: DoubleEndedIterator<Item = BoxedAction> + ExactSizeIterator + Debug,
    {
        let actions = actions.into_iter();
        let len = actions.len();

        if len == 0 {
            return;
        }

        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            warn!("Cannot add actions {actions:?} to non-existent agent {agent}.");
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            warn!(
                "Cannot add actions {actions:?} to agent {agent} due to missing component {}.",
                std::any::type_name::<ActionQueue>()
            );
            return;
        };

        debug!("Adding actions {actions:?} for agent {agent} with {config:?}.");
        action_queue.reserve(len);

        match config.order {
            AddOrder::Back => {
                for mut action in actions {
                    action.on_add(agent, world);

                    let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                        warn!(
                            "Cannot enqueue action {action:?} to non-existent agent {agent}. \
                            Action is therefore dropped immediately."
                        );
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Skipped);
                        return;
                    };

                    let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                        warn!(
                            "Cannot enqueue action {action:?} to agent {agent} due to missing component {}. \
                            Action is therefore dropped immediately.", std::any::type_name::<ActionQueue>()
                        );
                        action.on_remove(agent.into(), world);
                        action.on_drop(agent.into(), world, DropReason::Skipped);
                        return;
                    };

                    action_queue.push_back(action);
                }
            }
            AddOrder::Front => {
                for mut action in actions.rev() {
                    action.on_add(agent, world);

                    let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                        warn!(
                            "Cannot enqueue action {action:?} to non-existent agent {agent}. \
                            Action is therefore dropped immediately."
                        );
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Skipped);
                        return;
                    };

                    let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                        warn!(
                            "Cannot enqueue action {action:?} to agent {agent} due to missing component {}. \
                            Action is therefore dropped immediately.", std::any::type_name::<ActionQueue>()
                        );
                        action.on_remove(agent.into(), world);
                        action.on_drop(agent.into(), world, DropReason::Skipped);
                        return;
                    };

                    action_queue.push_front(action);
                }
            }
        }

        if config.start {
            let Some(current_action) = world.get::<CurrentAction>(agent) else {
                warn!(
                    "Could not start next action for agent {agent} due to missing component {}.",
                    std::any::type_name::<CurrentAction>()
                );
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
            warn!("Cannot execute actions for non-existent agent {agent}.");
            return;
        };

        let Some(current_action) = agent_ref.get::<CurrentAction>() else {
            warn!(
                "Cannot execute actions for agent {agent} due to missing component {}.",
                std::any::type_name::<CurrentAction>()
            );
            return;
        };

        if current_action.is_none() {
            debug!("Executing actions for agent {agent}.");
            Self::start_next_action(agent, world);
        }
    }

    /// [`Stops`](Action::on_stop) the current [`action`](Action) for `agent` with specified `reason`.
    pub fn stop_current_action(agent: Entity, reason: StopReason, world: &mut World) {
        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            warn!(
                "Cannot stop current action for non-existent agent {agent} with reason {reason:?}."
            );
            return;
        };

        let Some(mut current_action) = agent_ref.get_mut::<CurrentAction>() else {
            warn!(
                "Cannot stop current action for agent {agent} with reason {reason:?} \
                due to missing component {}.",
                std::any::type_name::<CurrentAction>()
            );
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
                        warn!(
                            "Cannot enqueue paused action {action:?} to non-existent agent {agent}. \
                            Action is therefore dropped immediately."
                        );
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Skipped);
                        return;
                    };

                    let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                        warn!(
                            "Cannot enqueue paused action {action:?} to agent {agent} due to missing component {}. \
                            Action is therefore dropped immediately.", std::any::type_name::<ActionQueue>()
                        );
                        action.on_remove(agent.into(), world);
                        action.on_drop(agent.into(), world, DropReason::Skipped);
                        return;
                    };

                    action_queue.push_front(action);
                }
            }
        }
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue for `agent`.
    ///
    /// This will loop until any next action is not immediately finished or the queue is empty.
    /// Since this may trigger an infinite loop, a counter is used in debug build
    /// that panics when reaching a sufficient target.
    ///
    /// The loop will also break if `agent` already has a current action.
    /// This is likely a user error, and so a warning will be emitted.
    pub fn start_next_action(agent: Entity, world: &mut World) {
        #[cfg(debug_assertions)]
        let mut counter: u16 = 0;

        loop {
            let Some(mut agent_ref) = world.get_entity_mut(agent) else {
                warn!("Cannot start next action for non-existent agent {agent}.");
                break;
            };

            let Some(current_action) = agent_ref.get::<CurrentAction>() else {
                warn!(
                    "Cannot start next action for agent {agent} due to missing component {}.",
                    std::any::type_name::<CurrentAction>()
                );
                break;
            };

            if let Some(action) = current_action.0.as_ref() {
                warn!(
                    "Cannot start next action for agent {agent} \
                    as it already has current action {action:?}."
                );
                break;
            }

            let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
                warn!(
                    "Cannot start next action for agent {agent} due to missing component {}.",
                    std::any::type_name::<ActionQueue>()
                );
                break;
            };

            let Some(mut action) = action_queue.pop_front() else {
                break;
            };

            debug!("Starting action {action:?} for agent {agent}.");
            if !action.on_start(agent, world) {
                match world.get_mut::<CurrentAction>(agent) {
                    Some(mut current_action) => {
                        current_action.0 = Some(action);
                    }
                    None => {
                        debug!("Canceling action {action:?} due to missing agent {agent}.");
                        action.on_stop(None, world, StopReason::Canceled);
                        action.on_remove(None, world);
                        action.on_drop(None, world, DropReason::Done);
                    }
                }
                break;
            };

            debug!("Finishing action {action:?} for agent {agent}.");
            let agent = world.get_entity(agent).map(|_| agent);
            action.on_stop(agent, world, StopReason::Finished);
            action.on_remove(agent, world);
            action.on_drop(agent, world, DropReason::Done);

            if agent.is_none() {
                break;
            }

            #[cfg(debug_assertions)]
            {
                counter += 1;
                if counter == u16::MAX {
                    panic!("infinite loop detected in starting next action");
                }
            }
        }
    }

    /// Skips the next [`action`](Action) in the queue for `agent`.
    pub fn skip_next_action(agent: Entity, world: &mut World) {
        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            warn!("Cannot skip next action for non-existent agent {agent}.");
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            warn!(
                "Cannot skip next action for agent {agent} due to missing component {}.",
                std::any::type_name::<ActionQueue>()
            );
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
            warn!("Cannot clear actions for non-existent agent {agent}.");
            return;
        };

        let Some(mut current_action) = agent_ref.get_mut::<CurrentAction>() else {
            warn!(
                "Cannot clear current action for agent {agent} due to missing component {}.",
                std::any::type_name::<CurrentAction>()
            );
            return;
        };

        if let Some(mut action) = current_action.take() {
            debug!("Clearing current action {action:?} for agent {agent}.");
            action.on_stop(agent.into(), world, StopReason::Canceled);
            action.on_remove(agent.into(), world);
            action.on_drop(agent.into(), world, DropReason::Cleared);
        }

        let Some(mut agent_ref) = world.get_entity_mut(agent) else {
            warn!("Cannot clear action queue for non-existent agent {agent}.");
            return;
        };

        let Some(mut action_queue) = agent_ref.get_mut::<ActionQueue>() else {
            warn!(
                "Cannot clear action queue for agent {agent} due to missing component {}.",
                std::any::type_name::<ActionQueue>()
            );
            return;
        };

        if action_queue.is_empty() {
            return;
        }

        debug!("Clearing action queue {:?} for {agent}.", **action_queue);
        let actions = std::mem::take(&mut action_queue.0);
        for mut action in actions {
            action.on_remove(agent.into(), world);
            action.on_drop(agent.into(), world, DropReason::Cleared);
        }
    }
}
