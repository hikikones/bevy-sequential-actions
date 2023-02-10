use crate::*;

impl<'a> ActionsProxy<'a> for World {
    type Modifier = AgentWorldActions<'a>;

    fn actions(&'a mut self, agent: Entity) -> AgentWorldActions<'a> {
        AgentWorldActions {
            agent,
            config: AddConfig::default(),
            world: self,
        }
    }
}

/// Modify actions using [`World`].
pub struct AgentWorldActions<'w> {
    agent: Entity,
    config: AddConfig,
    world: &'w mut World,
}

impl ModifyActions for AgentWorldActions<'_> {
    fn config(&mut self, config: AddConfig) -> &mut Self {
        self.config = config;
        self
    }

    fn add(&mut self, action: impl Into<ActionType>) -> &mut Self {
        self.world.add_action(self.agent, self.config, action);
        self
    }

    fn next(&mut self) -> &mut Self {
        self.world.next_action(self.agent);
        self
    }

    fn cancel(&mut self) -> &mut Self {
        self.world.cancel_action(self.agent);
        self
    }

    fn pause(&mut self) -> &mut Self {
        self.world.pause_action(self.agent);
        self
    }

    fn skip(&mut self) -> &mut Self {
        self.world.skip_action(self.agent);
        self
    }

    fn clear(&mut self) -> &mut Self {
        self.world.clear_actions(self.agent);
        self
    }
}

pub(super) trait ModifyActionsWorldExt {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: impl Into<ActionType>);
    fn next_action(&mut self, agent: Entity);
    fn finish_action(&mut self, agent: Entity);
    fn cancel_action(&mut self, agent: Entity);
    fn pause_action(&mut self, agent: Entity);
    fn skip_action(&mut self, agent: Entity);
    fn clear_actions(&mut self, agent: Entity);
}

impl ModifyActionsWorldExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: impl Into<ActionType>) {
        match Into::<ActionType>::into(action) {
            ActionType::Single(action) => {
                self.action_queue(agent).push(
                    config.order,
                    (InternalActionType::Single(action), config.repeat),
                );
            }
            ActionType::Sequence(actions) => {
                let mut queue = self.action_queue(agent);

                match config.order {
                    AddOrder::Back => {
                        for action in actions {
                            queue.push_back((InternalActionType::Single(action), config.repeat));
                        }
                    }
                    AddOrder::Front => {
                        for action in actions.rev() {
                            queue.push_front((InternalActionType::Single(action), config.repeat));
                        }
                    }
                }
            }
            ActionType::Parallel(actions) => {
                let actions = actions.collect::<Box<_>>();

                if !actions.is_empty() {
                    self.action_queue(agent).push(
                        config.order,
                        (InternalActionType::Parallel(actions), config.repeat),
                    );
                }
            }
            ActionType::Linked(actions) => {
                let actions = actions.filter(|a| !a.is_empty()).collect::<Box<[_]>>();

                if !actions.is_empty() {
                    self.action_queue(agent).push(
                        config.order,
                        (InternalActionType::Linked(actions, 0), config.repeat),
                    );
                }
            }
        }

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn next_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        self.start_next_action(agent);
    }

    fn finish_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Finished);
        self.start_next_action(agent);
    }

    fn cancel_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        self.start_next_action(agent);
    }

    fn pause_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Paused);
    }

    fn skip_action(&mut self, agent: Entity) {
        if let Some((action, mut repeat)) = self.pop_next_action(agent) {
            if repeat.process() {
                self.action_queue(agent).push_back((action, repeat));
            }
        }
    }

    fn clear_actions(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        self.action_queue(agent).clear();
    }
}

trait WorldActionsExt {
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason);
    fn start_next_action(&mut self, agent: Entity);
    fn take_current_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn pop_next_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue>;
    fn has_current_action(&self, agent: Entity) -> bool;
}

impl WorldActionsExt for World {
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some((mut current_action, mut repeat)) = self.take_current_action(agent) {
            self.get_mut::<ActionFinished>(agent)
                .unwrap()
                .bypass_change_detection()
                .reset_counts();

            match &mut current_action {
                InternalActionType::Single(action) => {
                    action.on_stop(agent, self, reason);

                    match reason {
                        StopReason::Finished | StopReason::Canceled => {
                            if repeat.process() {
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Paused => {
                            self.action_queue(agent)
                                .push_front((current_action, repeat));
                        }
                    }
                }
                InternalActionType::Parallel(actions) => {
                    actions
                        .iter_mut()
                        .for_each(|action| action.on_stop(agent, self, reason));

                    match reason {
                        StopReason::Finished | StopReason::Canceled => {
                            if repeat.process() {
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Paused => {
                            self.action_queue(agent)
                                .push_front((current_action, repeat));
                        }
                    }
                }
                InternalActionType::Linked(actions, index) => {
                    for action in actions[*index].iter_mut() {
                        action.on_stop(agent, self, reason);
                    }

                    match reason {
                        StopReason::Finished => {
                            *index += 1;

                            if *index < actions.len() {
                                self.action_queue(agent)
                                    .push_front((current_action, repeat));
                            } else if *index == actions.len() && repeat.process() {
                                *index = 0;
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Canceled => {
                            if repeat.process() {
                                *index = 0;
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Paused => {
                            self.action_queue(agent)
                                .push_front((current_action, repeat));
                        }
                    }
                }
            }
        }
    }

    fn start_next_action(&mut self, agent: Entity) {
        if let Some((mut next_action, repeat)) = self.pop_next_action(agent) {
            let mut commands = ActionCommands::new();

            match &mut next_action {
                InternalActionType::Single(action) => action.on_start(agent, self, &mut commands),
                InternalActionType::Parallel(actions) => actions
                    .iter_mut()
                    .for_each(|action| action.on_start(agent, self, &mut commands)),
                InternalActionType::Linked(actions, index) => {
                    for action in actions[*index].iter_mut() {
                        action.on_start(agent, self, &mut commands)
                    }
                }
            }

            self.get_mut::<CurrentAction>(agent).unwrap().0 = Some((next_action, repeat));

            commands.apply(self);
        }
    }

    fn take_current_action(&mut self, agent: Entity) -> Option<ActionTuple> {
        self.get_mut::<CurrentAction>(agent).unwrap().take()
    }

    fn pop_next_action(&mut self, agent: Entity) -> Option<ActionTuple> {
        self.action_queue(agent).pop_front()
    }

    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue> {
        self.get_mut::<ActionQueue>(agent).unwrap()
    }

    fn has_current_action(&self, agent: Entity) -> bool {
        self.get::<CurrentAction>(agent).unwrap().is_some()
    }
}
