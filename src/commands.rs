use crate::*;

impl<'c, 'w: 'c, 's: 'c> ActionsProxy<'c> for Commands<'w, 's> {
    type Modifier = AgentCommands<'c, 'w, 's>;

    fn actions(&'c mut self, agent: Entity) -> AgentCommands<'c, 'w, 's> {
        AgentCommands {
            agent,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Modify actions using [`Commands`].
pub struct AgentCommands<'c, 'w, 's> {
    agent: Entity,
    config: AddConfig,
    commands: &'c mut Commands<'w, 's>,
}

impl ModifyActions for AgentCommands<'_, '_, '_> {
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
        let agent = self.agent;
        let config = self.config;
        let action = action.into();

        self.commands.add(move |world: &mut World| {
            ActionHandler::add(agent, config, action, world);
        });

        self
    }

    fn add_many<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator,
    {
        let agent = self.agent;
        let config = self.config;

        self.commands.add(move |world: &mut World| {
            ActionHandler::add_many(agent, config, actions, world);
        });

        self
    }

    fn execute(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            ActionHandler::execute(agent, world);
        });

        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            ActionHandler::stop_current(agent, StopReason::Canceled, world);
            ActionHandler::start_next(agent, world);
        });

        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            ActionHandler::stop_current(agent, StopReason::Canceled, world);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            ActionHandler::stop_current(agent, StopReason::Paused, world);
        });

        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            ActionHandler::skip_next(agent, world);
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            ActionHandler::clear(agent, world);
        });

        self
    }
}
