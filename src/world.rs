use crate::*;

impl<'a> ActionsProxy<'a> for World {
    type Modifier = AgentActions<'a>;

    fn actions(&'a mut self, agent: Entity) -> Self::Modifier {
        Self::Modifier {
            agent,
            config: AddConfig::default(),
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
    fn config(&mut self, config: AddConfig) -> &mut Self {
        self.config = config;
        self
    }

    fn start(&mut self, start: bool) -> &mut Self {
        self.config.start = start;
        self
    }

    fn order(&mut self, order: AddOrder) -> &mut Self {
        self.config.order = order;
        self
    }

    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self {
        ActionHandler::add(self.agent, self.config, action, self.world);
        self
    }

    fn add_many<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator,
    {
        ActionHandler::add_many(self.agent, self.config, actions, self.world);
        self
    }

    fn execute(&mut self) -> &mut Self {
        ActionHandler::execute(self.agent, self.world);
        self
    }

    fn next(&mut self) -> &mut Self {
        ActionHandler::stop_current(self.agent, StopReason::Canceled, self.world);
        ActionHandler::start_next(self.agent, self.world);
        self
    }

    fn cancel(&mut self) -> &mut Self {
        ActionHandler::stop_current(self.agent, StopReason::Canceled, self.world);
        self
    }

    fn pause(&mut self) -> &mut Self {
        ActionHandler::stop_current(self.agent, StopReason::Paused, self.world);
        self
    }

    fn skip(&mut self) -> &mut Self {
        ActionHandler::skip_next(self.agent, self.world);
        self
    }

    fn clear(&mut self) -> &mut Self {
        ActionHandler::clear(self.agent, self.world);
        self
    }
}

impl ActionHandler {
    /// Adds a single [`action`](Action) to `agent` with specified `config`.
    pub fn add(
        agent: Entity,
        config: AddConfig,
        action: impl Into<BoxedAction>,
        world: &mut World,
    ) {
        let mut action = action.into();
        action.on_add(agent, world);

        let mut action_queue = world.get_mut::<ActionQueue>(agent).unwrap();
        match config.order {
            AddOrder::Back => action_queue.push_back(action),
            AddOrder::Front => action_queue.push_front(action),
        }

        if config.start && world.get::<CurrentAction>(agent).unwrap().is_none() {
            Self::start_next(agent, world);
        }
    }

    /// Adds a collection of actions to `agent` with specified `config`.
    pub fn add_many<I>(agent: Entity, config: AddConfig, actions: I, world: &mut World)
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator,
    {
        match config.order {
            AddOrder::Back => {
                for mut action in actions {
                    action.on_add(agent, world);
                    world
                        .get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_back(action);
                }
            }
            AddOrder::Front => {
                for mut action in actions.into_iter().rev() {
                    action.on_add(agent, world);
                    world
                        .get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_front(action);
                }
            }
        }

        if config.start && world.get::<CurrentAction>(agent).unwrap().is_none() {
            Self::start_next(agent, world);
        }
    }

    /// [`Starts`](Action::on_start) the next [`action`](Action) in the queue for `agent`,
    /// but only if there is no current action.
    pub fn execute(agent: Entity, world: &mut World) {
        if world.get::<CurrentAction>(agent).unwrap().is_none() {
            Self::start_next(agent, world);
        }
    }

    /// [`Stops`](Action::on_stop) the current [`action`](Action) for `agent` with specified `reason`.
    pub fn stop_current(agent: Entity, reason: StopReason, world: &mut World) {
        if let Some(mut action) = world.get_mut::<CurrentAction>(agent).unwrap().take() {
            action.on_stop(agent, world, reason);

            match reason {
                StopReason::Finished => {
                    action.on_remove(agent, world);
                    action.on_drop(agent, world, DropReason::Done);
                    Self::start_next(agent, world);
                }
                StopReason::Canceled => {
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
    pub fn start_next(agent: Entity, world: &mut World) {
        if let Some(mut next_action) = world.get_mut::<ActionQueue>(agent).unwrap().pop_front() {
            if next_action.on_start(agent, world) {
                next_action.on_stop(agent, world, StopReason::Finished);
                next_action.on_remove(agent, world);
                next_action.on_drop(agent, world, DropReason::Done);
                Self::start_next(agent, world);
                return;
            }

            if let Some(mut current_action) = world.get_mut::<CurrentAction>(agent) {
                current_action.0 = Some(next_action);
            }
        }
    }

    /// Skips the next [`action`](Action) in the queue for `agent`.
    pub fn skip_next(agent: Entity, world: &mut World) {
        if let Some(mut action) = world.get_mut::<ActionQueue>(agent).unwrap().pop_front() {
            action.on_remove(agent, world);
            action.on_drop(agent, world, DropReason::Skipped);
        }
    }

    /// Clears the action queue for `agent`.
    ///
    /// Current action is [`stopped`](Action::on_stop) as [`canceled`](StopReason::Canceled).
    pub fn clear(agent: Entity, world: &mut World) {
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
