use super::*;

impl ActionsProxy for Commands<'_, '_> {
    fn actions(&mut self, agent: Entity) -> impl ManageActions {
        AgentCommands {
            agent,
            config: AddConfig::default(),
            commands: self,
        }
    }
}

/// Manage actions using [`Commands`].
pub struct AgentCommands<'c, 'w, 's> {
    agent: Entity,
    config: AddConfig,
    commands: &'c mut Commands<'w, 's>,
}

impl ManageActions for AgentCommands<'_, '_, '_> {
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

    fn add(&mut self, action: impl IntoBoxedActions) -> &mut Self {
        let mut actions = action.into_boxed_actions();

        match actions.len() {
            0 => {}
            1 => {
                let agent = self.agent;
                let config = self.config;
                let action = actions.next().unwrap();
                self.commands.queue(move |world: &mut World| {
                    SequentialActionsPlugin::add_action(agent, config, action, world);
                });
            }
            _ => {
                let agent = self.agent;
                let config = self.config;
                self.commands.queue(move |world: &mut World| {
                    SequentialActionsPlugin::add_actions(agent, config, actions, world);
                });
            }
        }

        self
    }

    fn execute(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.queue(move |world: &mut World| {
            SequentialActionsPlugin::execute_actions(agent, world);
        });

        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.queue(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
            SequentialActionsPlugin::start_next_action(agent, world);
        });

        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.queue(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Canceled, world);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.queue(move |world: &mut World| {
            SequentialActionsPlugin::stop_current_action(agent, StopReason::Paused, world);
        });

        self
    }

    fn skip(&mut self, n: usize) -> &mut Self {
        let agent = self.agent;

        self.commands.queue(move |world: &mut World| {
            SequentialActionsPlugin::skip_actions(agent, n, world);
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.queue(move |world: &mut World| {
            SequentialActionsPlugin::clear_actions(agent, world);
        });

        self
    }
}
