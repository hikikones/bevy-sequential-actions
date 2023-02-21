use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[allow(clippy::type_complexity)]
pub struct ActionCommands(Vec<Box<dyn FnOnce(&mut World)>>);

impl ActionCommands {
    pub(super) fn new() -> Self {
        Self(Vec::new())
    }

    pub(super) fn apply(self, world: &mut World) {
        for cmd in self.0 {
            cmd(world);
        }
    }

    /// Adds a custom command to the command queue.
    /// Used for modifying actions in a deferred way using [`World`] inside the [`Action`] trait.
    pub fn add(&mut self, f: impl FnOnce(&mut World) + 'static) -> &mut Self {
        self.0.push(Box::new(f));
        self
    }
}

impl<'a> ActionsProxy<'a> for ActionCommands {
    type Modifier = AgentActions<'a>;

    fn actions(&'a mut self, agent: Entity) -> AgentActions<'a> {
        AgentActions {
            agent,
            config: AddConfig::new(),
            commands: self,
        }
    }
}

/// Modify actions using [`ActionCommands`].
pub struct AgentActions<'a> {
    agent: Entity,
    config: AddConfig,
    commands: &'a mut ActionCommands,
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

    fn repeat(&mut self, repeat: Repeat) -> &mut Self {
        self.config.repeat = repeat;
        self
    }

    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self {
        let agent = self.agent;
        let config = self.config;
        let action = action.into();

        self.commands.add(move |world| {
            world.add_action(agent, config, action);
        });

        self
    }

    fn add_sequence(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
        let agent = self.agent;
        let config = self.config;

        self.commands.add(move |world: &mut World| {
            world.add_actions(agent, config, actions);
        });

        self
    }

    fn add_parallel(
        &mut self,
        actions: impl Iterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
        let agent = self.agent;
        let config = self.config;

        self.commands.add(move |world: &mut World| {
            world.add_parallel_actions(agent, config, actions);
        });

        self
    }

    fn add_linked(
        &mut self,
        f: impl FnOnce(&mut LinkedActionsBuilder) + Send + Sync + 'static,
    ) -> &mut Self {
        let agent = self.agent;
        let config = self.config;

        self.commands.add(move |world: &mut World| {
            world.add_linked_actions(agent, config, f);
        });

        self
    }

    fn execute(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world: &mut World| {
            world.execute_actions(agent);
        });

        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.next_action(agent);
        });

        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.cancel_action(agent);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.pause_action(agent);
        });

        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.skip_action(agent);
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;

        self.commands.add(move |world| {
            world.clear_actions(agent);
        });

        self
    }
}
