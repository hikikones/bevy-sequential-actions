use bevy_ecs::system::{Command, CommandQueue};

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

    fn add_many(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
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
        self.world.stop_action(self.agent, StopReason::Canceled);
        self
    }

    fn pause(&mut self) -> &mut Self {
        self.world.stop_action(self.agent, StopReason::Paused);
        self
    }

    fn skip(&mut self) -> &mut Self {
        self.world.skip_action(self.agent);
        self
    }

    fn clear(&mut self) -> &mut Self {
        self.world.clear_actions(self.agent);
        self
    }
}

#[derive(Default, Resource, Deref, DerefMut)]
pub(super) struct DeferredActions(CommandQueue);

impl<'w> DeferredActionsProxy<'w> for World {
    type Modifier = DeferredAgentActions<'w>;

    fn deferred_actions(&'w mut self, agent: Entity) -> Self::Modifier {
        Self::Modifier {
            agent,
            config: AddConfig::new(),
            world: self,
        }
    }
}

/// Modify actions using [`World`] in a deferred way.
pub struct DeferredAgentActions<'w> {
    agent: Entity,
    config: AddConfig,
    world: &'w mut World,
}

impl<'w> DeferredAgentActions<'w> {
    /// Adds a custom [`Command`].
    pub fn custom(&mut self, command: impl Command) -> &mut Self {
        self.world.push_deferred_action(command);
        self
    }
}

impl ModifyActions for DeferredAgentActions<'_> {
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

        self.world.push_deferred_action(move |world: &mut World| {
            world.add_action(agent, config, action);
        });

        self
    }

    fn add_many(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
        let agent = self.agent;
        let config = self.config;

        self.world.push_deferred_action(move |world: &mut World| {
            world.add_actions(agent, config, actions);
        });

        self
    }

    fn execute(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.execute_actions(agent);
        });

        self
    }

    fn next(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.next_action(agent);
        });

        self
    }

    fn cancel(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.stop_action(agent, StopReason::Canceled);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.stop_action(agent, StopReason::Paused);
        });

        self
    }

    fn skip(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.skip_action(agent);
        });

        self
    }

    fn clear(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.clear_actions(agent);
        });

        self
    }
}

pub(super) trait WorldActionsExt {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: BoxedAction);
    fn add_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl DoubleEndedIterator<Item = BoxedAction>,
    );
    fn execute_actions(&mut self, agent: Entity);
    fn next_action(&mut self, agent: Entity);
    fn stop_action(&mut self, agent: Entity, reason: StopReason);
    fn skip_action(&mut self, agent: Entity);
    fn clear_actions(&mut self, agent: Entity);
}

impl WorldActionsExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, mut action: BoxedAction) {
        action.on_add(agent, self);

        self.action_queue(agent).push(config.order, action);

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn add_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl DoubleEndedIterator<Item = BoxedAction>,
    ) {
        match config.order {
            AddOrder::Back => {
                for mut action in actions {
                    action.on_add(agent, self);
                    self.action_queue(agent).push_back(action);
                }
            }
            AddOrder::Front => {
                for mut action in actions.rev() {
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
        self.stop_action(agent, StopReason::Canceled);
        self.start_next_action(agent);
    }

    fn stop_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some(mut action) = self.take_current_action(agent) {
            action.on_stop(agent, self, reason);

            match reason {
                StopReason::Finished => {
                    action.on_remove(agent, self);
                    self.start_next_action(agent);
                }
                StopReason::Canceled => {
                    action.on_remove(agent, self);
                }
                StopReason::Paused => {
                    self.action_queue(agent).push_front(action);
                }
            }
        }
    }

    fn skip_action(&mut self, agent: Entity) {
        if let Some(action) = self.pop_next_action(agent) {
            action.on_remove(agent, self);
        }
    }

    fn clear_actions(&mut self, agent: Entity) {
        self.stop_action(agent, StopReason::Canceled);

        let actions = std::mem::take(&mut self.action_queue(agent).0);
        for action in actions {
            action.on_remove(agent, self);
        }
    }
}

trait WorldHelperExt {
    fn start_next_action(&mut self, agent: Entity);
    fn take_current_action(&mut self, agent: Entity) -> Option<BoxedAction>;
    fn pop_next_action(&mut self, agent: Entity) -> Option<BoxedAction>;
    fn current_action(&mut self, agent: Entity) -> Mut<CurrentAction>;
    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue>;
    fn push_deferred_action(&mut self, command: impl Command);
    fn apply_deferred_actions(&mut self);
    fn has_current_action(&self, agent: Entity) -> bool;
}

impl WorldHelperExt for World {
    fn start_next_action(&mut self, agent: Entity) {
        if let Some(mut action) = self.pop_next_action(agent) {
            action.on_start(agent, self);
            self.current_action(agent).0 = Some(action);
            self.apply_deferred_actions();
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

    fn push_deferred_action(&mut self, command: impl Command) {
        self.resource_mut::<DeferredActions>().push(command);
    }

    fn apply_deferred_actions(&mut self) {
        let mut actions = std::mem::take(&mut self.resource_mut::<DeferredActions>().0);
        actions.apply(self);
    }

    fn has_current_action(&self, agent: Entity) -> bool {
        self.get::<CurrentAction>(agent).unwrap().is_some()
    }
}
