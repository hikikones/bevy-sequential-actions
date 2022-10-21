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

    fn add(&mut self, action: impl IntoBoxedAction) -> &mut Self {
        self.world.add_action(self.agent, self.config, action);
        self
    }

    fn add_many(&mut self, mode: ExecutionMode, actions: impl BoxedActionIter) -> &mut Self {
        self.world
            .add_actions(self.agent, self.config, mode, actions);
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
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: impl IntoBoxedAction);
    fn add_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        mode: ExecutionMode,
        actions: impl BoxedActionIter,
    );
    fn next_action(&mut self, agent: Entity);
    fn finish_action(&mut self, agent: Entity);
    fn cancel_action(&mut self, agent: Entity);
    fn pause_action(&mut self, agent: Entity);
    fn skip_action(&mut self, agent: Entity);
    fn clear_actions(&mut self, agent: Entity);
}

impl ModifyActionsWorldExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: impl IntoBoxedAction) {
        let action_tuple = (
            ActionType::Single((action.into_boxed(), None)),
            config.into(),
        );
        let mut queue = self.action_queue(agent);

        match config.order {
            AddOrder::Back => queue.push_back(action_tuple),
            AddOrder::Front => queue.push_front(action_tuple),
        }

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn add_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        mode: ExecutionMode,
        actions: impl BoxedActionIter,
    ) {
        let mut queue = self.action_queue(agent);

        match mode {
            ExecutionMode::Sequential => match config.order {
                AddOrder::Back => {
                    for action in actions {
                        queue.push_back((ActionType::Single((action, None)), config.into()));
                    }
                }
                AddOrder::Front => {
                    for action in actions.rev() {
                        queue.push_front((ActionType::Single((action, None)), config.into()));
                    }
                }
            },
            ExecutionMode::Parallel => {
                let action = ActionType::Multiple(actions.map(|action| (action, None)).collect());
                match config.order {
                    AddOrder::Back => queue.push_back((action, config.into())),
                    AddOrder::Front => queue.push_front((action, config.into())),
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
        if let Some((action, state)) = self.pop_next_action(agent) {
            self.handle_repeat_action(agent, action, state);
        }
    }

    fn clear_actions(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        self.action_queue(agent).clear();
    }
}

trait WorldActionsExt {
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason);
    fn handle_repeat_action(&mut self, agent: Entity, action: ActionType, state: ActionState);
    fn start_next_action(&mut self, agent: Entity);
    fn take_current_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn pop_next_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue>;
    fn has_current_action(&self, agent: Entity) -> bool;
}

impl WorldActionsExt for World {
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some((mut action, state)) = self.take_current_action(agent) {
            match &mut action {
                ActionType::Single((action, e)) => {
                    let e = e.take().unwrap();
                    action.on_stop(ActionEntities(agent, e), self, reason);
                    self.despawn(e);
                }
                ActionType::Multiple(actions) => {
                    for (action, e) in actions.iter_mut() {
                        let e = e.take().unwrap();
                        action.on_stop(ActionEntities(agent, e), self, reason);
                        self.despawn(e);
                    }
                }
            }

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    self.handle_repeat_action(agent, action, state);
                }
                StopReason::Paused => {
                    self.action_queue(agent).push_front((action, state));
                }
            }
        }
    }

    fn handle_repeat_action(&mut self, agent: Entity, action: ActionType, mut state: ActionState) {
        match &mut state.repeat {
            Repeat::Amount(n) => {
                if *n > 0 {
                    *n -= 1;
                    self.action_queue(agent).push_back((action, state));
                }
            }
            Repeat::Forever => {
                self.action_queue(agent).push_back((action, state));
            }
        }
    }

    fn start_next_action(&mut self, agent: Entity) {
        if let Some((mut action, state)) = self.pop_next_action(agent) {
            let mut commands = ActionCommands::new();

            match &mut action {
                ActionType::Single((action, e)) => {
                    let e = *e.insert(
                        self.spawn()
                            .insert_bundle((ActionFinished(false), ActionAgent(agent)))
                            .id(),
                    );
                    action.on_start(ActionEntities(agent, e), self, &mut commands);
                }
                ActionType::Multiple(actions) => {
                    for (action, e) in actions.iter_mut() {
                        let e = *e.insert(
                            self.spawn()
                                .insert_bundle((ActionFinished(false), ActionAgent(agent)))
                                .id(),
                        );
                        action.on_start(ActionEntities(agent, e), self, &mut commands);
                    }
                }
            }

            if let Some(mut agent) = self.get_entity_mut(agent) {
                agent.get_mut::<CurrentAction>().unwrap().0 = Some((action, state));
            } else {
                match action {
                    ActionType::Single((_, e)) => {
                        self.despawn(e.unwrap());
                    }
                    ActionType::Multiple(actions) => {
                        for (_, e) in actions.iter() {
                            self.despawn(e.unwrap());
                        }
                    }
                }
            }

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
