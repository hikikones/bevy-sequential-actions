use super::*;

impl ModifyActionsExt for EntityCommands<'_> {
    fn add_action_with_config(&mut self, config: AddConfig, action: impl Action) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::add_action(agent, config, action, world);
        });

        self
    }

    fn add_actions_with_config<I>(&mut self, config: AddConfig, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator + ExactSizeIterator + Debug,
    {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::add_actions(agent, config, actions, world);
        });

        self
    }

    fn execute_actions(&mut self) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::execute_actions(agent, world);
        });

        self
    }

    fn next_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
            SequentialActionsPlugin::start_next_action(agent, world);
        });

        self
    }

    fn cancel_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
        });

        self
    }

    fn pause_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Paused, world);
        });

        self
    }

    fn skip_next_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::skip_next_action(agent, world);
        });

        self
    }

    fn clear_actions(&mut self) -> &mut Self {
        let agent = self.id();

        self.commands().add(move |world: &mut World| {
            SequentialActionsPlugin::clear_actions(agent, world);
        });

        self
    }
}

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

#[deprecated(
    since = "0.12.0",
    note = "Replaced by ModifyActionsExt trait implemented for EntityCommands and EntityWorldMut."
)]
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
            SequentialActionsPlugin::add_action(agent, config, action, world);
        });

        self
    }

    fn add_many<I>(&mut self, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator + ExactSizeIterator + Debug,
    {
        let agent = self.agent;
        let config = self.config;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::add_actions(agent, config, actions, world);
        });

        self
    }

    fn execute(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::execute_actions(agent, world);
        });

        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
            SequentialActionsPlugin::start_next_action(agent, world);
        });

        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Paused, world);
        });

        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::skip_next_action(agent, world);
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            SequentialActionsPlugin::clear_actions(agent, world);
        });

        self
    }
}
