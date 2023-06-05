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

    fn add_many<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator,
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
    fn add_actions<I>(&mut self, agent: Entity, config: AddConfig, actions: I)
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator;
    fn execute_actions(&mut self, agent: Entity);
    fn next_action(&mut self, agent: Entity);
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason);
    fn skip_next_action(&mut self, agent: Entity);
    fn clear_actions(&mut self, agent: Entity);
}

impl WorldActionsExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, mut action: BoxedAction) {
        action.on_add(agent, self);

        let mut action_queue = self.get_mut::<ActionQueue>(agent).unwrap();
        match config.order {
            AddOrder::Back => action_queue.push_back(action),
            AddOrder::Front => action_queue.push_front(action),
        }

        if config.start && self.get::<CurrentAction>(agent).unwrap().is_none() {
            start_next_action(agent, self);
        }
    }

    fn add_actions<I>(&mut self, agent: Entity, config: AddConfig, actions: I)
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator,
    {
        match config.order {
            AddOrder::Back => {
                for mut action in actions {
                    action.on_add(agent, self);
                    self.get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_back(action);
                }
            }
            AddOrder::Front => {
                for mut action in actions.into_iter().rev() {
                    action.on_add(agent, self);
                    self.get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_front(action);
                }
            }
        }

        if config.start && self.get::<CurrentAction>(agent).unwrap().is_none() {
            start_next_action(agent, self);
        }
    }

    fn execute_actions(&mut self, agent: Entity) {
        if self.get::<CurrentAction>(agent).unwrap().is_none() {
            start_next_action(agent, self);
        }
    }

    fn next_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        start_next_action(agent, self);
    }

    fn stop_current_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some(mut action) = self.get_mut::<CurrentAction>(agent).unwrap().take() {
            action.on_stop(agent, self, reason);

            match reason {
                StopReason::Finished => {
                    action.on_remove(agent, self);
                    action.on_drop(agent, self, DropReason::Done);
                    start_next_action(agent, self);
                }
                StopReason::Canceled => {
                    action.on_remove(agent, self);
                    action.on_drop(agent, self, DropReason::Done);
                }
                StopReason::Paused => {
                    self.get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_front(action);
                }
            }
        }
    }

    fn skip_next_action(&mut self, agent: Entity) {
        if let Some(mut action) = self.get_mut::<ActionQueue>(agent).unwrap().pop_front() {
            action.on_remove(agent, self);
            action.on_drop(agent, self, DropReason::Skipped);
        }
    }

    fn clear_actions(&mut self, agent: Entity) {
        if let Some(mut action) = self.get_mut::<CurrentAction>(agent).unwrap().take() {
            action.on_stop(agent, self, StopReason::Canceled);
            action.on_remove(agent, self);
            action.on_drop(agent, self, DropReason::Cleared);
        }

        for mut action in self
            .get_mut::<ActionQueue>(agent)
            .unwrap()
            .drain(..)
            .collect::<Vec<_>>()
        {
            action.on_remove(agent, self);
            action.on_drop(agent, self, DropReason::Cleared);
        }
    }
}

fn start_next_action(agent: Entity, world: &mut World) {
    let mut action_queue = world.get_mut::<ActionQueue>(agent).unwrap();

    if let Some(mut next_action) = action_queue.pop_front() {
        if next_action.on_start(agent, world).0 {
            next_action.on_stop(agent, world, StopReason::Finished);
            next_action.on_remove(agent, world);
            next_action.on_drop(agent, world, DropReason::Done);
            start_next_action(agent, world);
            return;
        }

        if let Some(mut current_action) = world.get_mut::<CurrentAction>(agent) {
            current_action.0 = Some(next_action);
        }
    }
}
