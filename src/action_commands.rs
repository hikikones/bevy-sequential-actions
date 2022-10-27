use crate::*;

/// Commands for modifying actions inside the [`Action`] trait.
#[allow(clippy::type_complexity)]
pub struct ActionCommands(Vec<Box<dyn FnOnce(&mut World)>>);

impl ActionCommands {
    fn push<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.0.push(Box::new(f));
    }

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
    pub fn add<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut World) + 'static,
    {
        self.push(f);
        self
    }
}

impl<'a> ActionsProxy<'a> for ActionCommands {
    type Modifier = AgentActions<'a>;

    fn actions(&'a mut self, agent: Entity) -> AgentActions<'a> {
        AgentActions {
            agent,
            config: AddConfig::default(),
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
    fn config(&mut self, config: AddConfig) -> &mut Self {
        self.config = config;
        self
    }

    fn add(&mut self, action: impl IntoBoxedAction) -> &mut Self {
        let agent = self.agent;
        let config = self.config;
        self.commands.push(move |world| {
            world.add_action(agent, config, action);
        });
        self
    }

    fn add_many(&mut self, mode: ExecutionMode, actions: impl BoxedActionIter) -> &mut Self {
        let agent = self.agent;
        let config = self.config;
        self.commands.push(move |world| {
            world.add_actions(agent, config, mode, actions);
        });
        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.push(move |world| {
            world.next_action(agent);
        });
        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.push(move |world| {
            world.cancel_action(agent);
        });
        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.push(move |world| {
            world.pause_action(agent);
        });
        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.push(move |world| {
            world.skip_action(agent);
        });
        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;
        self.commands.push(move |world| {
            world.clear_actions(agent);
        });
        self
    }
}
