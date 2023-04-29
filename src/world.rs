use crate::*;

impl<'a> ActionsProxy<'a> for World {
    type Modifier = AgentActions<'a>;

    fn actions(&'a mut self, agent: Entity) -> Self::Modifier {
        Self::Modifier {
            agent,
            config: AddConfig::new(),
            world: self,
        }
    }
}

/// Modify actions using [`World`].
pub struct AgentActions<'w> {
    agent: Entity,
    config: AddConfig,
    world: &'w mut World,
}

impl ModifyActions for AgentActions<'_> {
    fn start(&mut self, start: bool) -> &mut Self {
        self.config.start = start;
        self
    }

    fn order(&mut self, order: AddOrder) -> &mut Self {
        self.config.order = order;
        self
    }

    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self {
        self.world
            .add_action(self.agent, self.config, action.into());
        self
    }

    fn add_many<I>(
        &mut self,
        actions: impl IntoIterator<Item = BoxedAction, IntoIter = I> + Send + 'static,
    ) -> &mut Self
    where
        I: DoubleEndedIterator<Item = BoxedAction>,
    {
        self.world.add_actions(self.agent, self.config, actions);
        self
    }

    fn execute(&mut self) -> &mut Self {
        self.world.execute_actions(self.agent);
        self
    }

    fn next(&mut self) -> &mut Self {
        self.world.next_action(self.agent);
        self
    }

    fn cancel(&mut self) -> &mut Self {
        self.world
            .stop_current_action(self.agent, StopReason::Canceled);
        self
    }

    fn pause(&mut self) -> &mut Self {
        self.world
            .stop_current_action(self.agent, StopReason::Paused);
        self
    }

    fn skip(&mut self) -> &mut Self {
        self.world.skip_next_action(self.agent);
        self
    }

    fn clear(&mut self) -> &mut Self {
        self.world.clear_actions(self.agent);
        self
    }
}

pub(super) trait WorldActionsExt {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: BoxedAction);
    fn add_actions<I>(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl IntoIterator<Item = BoxedAction, IntoIter = I>,
    ) where
        I: DoubleEndedIterator<Item = BoxedAction>;
    fn execute_actions(&mut self, agent: Entity);
    fn next_action(&mut self, agent: Entity);
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason);
    fn skip_next_action(&mut self, agent: Entity);
    fn clear_actions(&mut self, agent: Entity);
}

impl WorldActionsExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, mut action: BoxedAction) {
        action.on_add(agent, self);

        match config.order {
            AddOrder::Back => self.action_queue(agent).push_back(action),
            AddOrder::Front => self.action_queue(agent).push_front(action),
        }

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn add_actions<I>(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl IntoIterator<Item = BoxedAction, IntoIter = I>,
    ) where
        I: DoubleEndedIterator<Item = BoxedAction>,
    {
        match config.order {
            AddOrder::Back => {
                for mut action in actions {
                    action.on_add(agent, self);
                    self.action_queue(agent).push_back(action);
                }
            }
            AddOrder::Front => {
                for mut action in actions.into_iter().rev() {
                    action.on_add(agent, self);
                    self.action_queue(agent).push_front(action);
                }
            }
        }

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn execute_actions(&mut self, agent: Entity) {
        if !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn next_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        self.start_next_action(agent);
    }

    fn stop_current_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some(mut action) = self.take_current_action(agent) {
            action.on_stop(agent, self, reason);

            match reason {
                StopReason::Finished => {
                    action.on_remove(agent, self);
                    action.on_drop(agent, self);
                    self.start_next_action(agent);
                }
                StopReason::Canceled => {
                    action.on_remove(agent, self);
                    action.on_drop(agent, self);
                }
                StopReason::Paused => {
                    self.action_queue(agent).push_front(action);
                }
            }
        }
    }

    fn skip_next_action(&mut self, agent: Entity) {
        if let Some(mut action) = self.pop_next_action(agent) {
            action.on_remove(agent, self);
            action.on_drop(agent, self);
        }
    }

    fn clear_actions(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);

        for mut action in self.action_queue(agent).drain(..).collect::<Vec<_>>() {
            action.on_remove(agent, self);
            action.on_drop(agent, self);
        }
    }
}

trait WorldHelperExt {
    fn start_next_action(&mut self, agent: Entity);
    fn take_current_action(&mut self, agent: Entity) -> Option<BoxedAction>;
    fn pop_next_action(&mut self, agent: Entity) -> Option<BoxedAction>;
    fn current_action(&mut self, agent: Entity) -> Mut<CurrentAction>;
    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue>;
    fn has_current_action(&self, agent: Entity) -> bool;
}

impl WorldHelperExt for World {
    fn start_next_action(&mut self, agent: Entity) {
        if let Some(mut action) = self.pop_next_action(agent) {
            let is_finished = action.on_start(agent, self);

            if is_finished {
                action.on_stop(agent, self, StopReason::Finished);
                action.on_remove(agent, self);
                action.on_drop(agent, self);
                self.start_next_action(agent);
                return;
            }

            if let Some(mut current) = self.get_mut::<CurrentAction>(agent) {
                current.0 = Some(action);
            }
        }
    }

    fn take_current_action(&mut self, agent: Entity) -> Option<BoxedAction> {
        self.get_mut::<CurrentAction>(agent).unwrap().take()
    }

    fn pop_next_action(&mut self, agent: Entity) -> Option<BoxedAction> {
        self.action_queue(agent).pop_front()
    }

    fn current_action(&mut self, agent: Entity) -> Mut<CurrentAction> {
        self.get_mut::<CurrentAction>(agent).unwrap()
    }

    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue> {
        self.get_mut::<ActionQueue>(agent).unwrap()
    }

    fn has_current_action(&self, agent: Entity) -> bool {
        self.get::<CurrentAction>(agent).unwrap().is_some()
    }
}
