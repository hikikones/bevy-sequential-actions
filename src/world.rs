use crate::*;

impl ActionsProxy for World {
    fn actions(&mut self, agent: Entity) -> impl ModifyActions {
        AgentActions {
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
        SequentialActionsPlugin::add_action(self.agent, self.config, action, self.world);
        self
    }

    fn add_many<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction>,
        I::IntoIter: DoubleEndedIterator,
    {
        SequentialActionsPlugin::add_actions(self.agent, self.config, actions, self.world);
        self
    }

    fn execute(&mut self) -> &mut Self {
        SequentialActionsPlugin::execute_actions(self.agent, self.world);
        self
    }

    fn next(&mut self) -> &mut Self {
        SequentialActionsPlugin::stop_current_action(self.agent, StopReason::Canceled, self.world);
        SequentialActionsPlugin::start_next_action(self.agent, self.world);
        self
    }

    fn cancel(&mut self) -> &mut Self {
        SequentialActionsPlugin::stop_current_action(self.agent, StopReason::Canceled, self.world);
        self
    }

    fn pause(&mut self) -> &mut Self {
        SequentialActionsPlugin::stop_current_action(self.agent, StopReason::Paused, self.world);
        self
    }

    fn skip(&mut self) -> &mut Self {
        SequentialActionsPlugin::skip_next_action(self.agent, self.world);
        self
    }

    fn clear(&mut self) -> &mut Self {
        SequentialActionsPlugin::clear_actions(self.agent, self.world);
        self
    }
}
