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
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoBoxedAction) -> Self {
        self.world.add_action(self.agent, self.config, action);
        self
    }

    fn add_many(self, mode: ExecutionMode, actions: impl BoxedActionIter) -> Self {
        self.world
            .add_actions(self.agent, self.config, mode, actions);
        self
    }

    fn next(self) -> Self {
        self.world.next_action(self.agent);
        self
    }

    fn cancel(self) -> Self {
        self.world.cancel_action(self.agent);
        self
    }

    fn pause(self) -> Self {
        self.world.pause_action(self.agent);
        self
    }

    fn skip(self) -> Self {
        self.world.skip_action(self.agent);
        self
    }

    fn clear(self) -> Self {
        self.world.clear_actions(self.agent);
        self
    }
}

pub(super) trait WorldActionsExt {
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
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason);
    fn handle_repeat_action(&mut self, agent: Entity, action: ActionType, state: ActionState);
    fn start_next_action(&mut self, agent: Entity);
    fn take_current_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn pop_next_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue>;
    fn has_current_action(&self, agent: Entity) -> bool;
}

impl WorldActionsExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: impl IntoBoxedAction) {
        let action_tuple = (ActionType::Single(action.into_boxed()), config.into());
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
                        queue.push_back((ActionType::Single(action), config.into()));
                    }
                }
                AddOrder::Front => {
                    for action in actions.rev() {
                        queue.push_front((ActionType::Single(action), config.into()));
                    }
                }
            },
            ExecutionMode::Parallel => {
                let action = ActionType::Multiple(actions.collect::<Box<[_]>>());
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

    fn stop_current_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some((mut action, state)) = self.take_current_action(agent) {
            match &mut action {
                ActionType::Single(action) => {
                    action.on_stop(agent, self, reason);
                }
                ActionType::Multiple(actions) => {
                    for action in actions.iter_mut() {
                        action.on_stop(agent, self, reason);
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

            let mut finished = self.get_mut::<ActionFinished>(agent).unwrap();
            if finished.count != 0 {
                finished.count = 0;
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
                ActionType::Single(action) => {
                    action.on_start(agent, self, &mut commands);
                }
                ActionType::Multiple(actions) => {
                    for action in actions.iter_mut() {
                        action.on_start(agent, self, &mut commands);
                    }
                }
            }

            if let Some(mut current) = self.get_mut::<CurrentAction>(agent) {
                **current = Some((action, state));
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
