use crate::*;

impl ModifyActionsExt for EntityWorldMut<'_> {
    fn add_action_with_config(&mut self, config: AddConfig, action: impl Action) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::add_action(agent, config, action, world);
        });

        self
    }

    fn add_actions_with_config<I>(&mut self, config: AddConfig, actions: I) -> &mut Self
    where
        I: IntoIterator<Item = BoxedAction> + Send + 'static,
        I::IntoIter: DoubleEndedIterator,
    {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::add_actions(agent, config, actions, world);
        });

        self
    }

    fn execute_actions(&mut self) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::execute_actions(agent, world);
        });

        self
    }

    fn next_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
            SequentialActionsPlugin::start_next_action(agent, world);
        });

        self
    }

    fn cancel_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
        });

        self
    }

    fn pause_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Paused, world);
        });

        self
    }

    fn skip_next_action(&mut self) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::skip_next_action(agent, world);
        });

        self
    }

    fn clear_actions(&mut self) -> &mut Self {
        let agent = self.id();

        self.world_scope(move |world| {
            SequentialActionsPlugin::clear_actions(agent, world);
        });

        self
    }
}

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

#[deprecated(
    since = "0.12.0",
    note = "Replaced by ModifyActionsExt trait implemented for EntityCommands and EntityWorldMut."
)]
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
