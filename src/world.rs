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

    fn repeat(&mut self, repeat: Repeat) -> &mut Self {
        self.config.repeat = repeat;
        self
    }

    fn add(&mut self, action: impl Into<BoxedAction>) -> &mut Self {
        self.world
            .add_action(self.agent, self.config, action.into());
        self
    }

    fn add_sequence(
        &mut self,
        actions: impl DoubleEndedIterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
        self.world.add_actions(self.agent, self.config, actions);
        self
    }

    fn add_parallel(
        &mut self,
        actions: impl Iterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
        self.world
            .add_parallel_actions(self.agent, self.config, actions);
        self
    }

    fn add_linked(
        &mut self,
        f: impl FnOnce(&mut LinkedActionsBuilder) + Send + Sync + 'static,
    ) -> &mut Self {
        self.world.add_linked_actions(self.agent, self.config, f);
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
        self.world.cancel_action(self.agent);
        self
    }

    fn pause(&mut self) -> &mut Self {
        self.world.pause_action(self.agent);
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

    fn repeat(&mut self, repeat: Repeat) -> &mut Self {
        self.config.repeat = repeat;
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

    fn add_sequence(
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

    fn add_parallel(
        &mut self,
        actions: impl Iterator<Item = BoxedAction> + Send + Sync + 'static,
    ) -> &mut Self {
        let agent = self.agent;
        let config = self.config;

        self.world.push_deferred_action(move |world: &mut World| {
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

        self.world.push_deferred_action(move |world: &mut World| {
            world.add_linked_actions(agent, config, f);
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
            world.cancel_action(agent);
        });

        self
    }

    fn pause(&mut self) -> &mut Self {
        let agent = self.agent;

        self.world.push_deferred_action(move |world: &mut World| {
            world.pause_action(agent);
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
    fn add_parallel_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl Iterator<Item = BoxedAction>,
    );
    fn add_linked_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl FnOnce(&mut LinkedActionsBuilder),
    );
    fn execute_actions(&mut self, agent: Entity);
    fn next_action(&mut self, agent: Entity);
    fn finish_action(&mut self, agent: Entity);
    fn cancel_action(&mut self, agent: Entity);
    fn pause_action(&mut self, agent: Entity);
    fn skip_action(&mut self, agent: Entity);
    fn clear_actions(&mut self, agent: Entity);
}

impl WorldActionsExt for World {
    fn add_action(&mut self, agent: Entity, config: AddConfig, action: BoxedAction) {
        self.action_queue(agent)
            .push(config.order, (ActionType::One(action), config.repeat));

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
        let mut queue = self.action_queue(agent);

        match config.order {
            AddOrder::Back => {
                for action in actions {
                    queue.push_back((ActionType::One(action), config.repeat));
                }
            }
            AddOrder::Front => {
                for action in actions.rev() {
                    queue.push_front((ActionType::One(action), config.repeat));
                }
            }
        }

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn add_parallel_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        actions: impl Iterator<Item = BoxedAction>,
    ) {
        let actions = actions.collect::<Box<_>>();

        if !actions.is_empty() {
            self.action_queue(agent)
                .push(config.order, (ActionType::Many(actions), config.repeat));
        }

        if config.start && !self.has_current_action(agent) {
            self.start_next_action(agent);
        }
    }

    fn add_linked_actions(
        &mut self,
        agent: Entity,
        config: AddConfig,
        f: impl FnOnce(&mut LinkedActionsBuilder),
    ) {
        let mut builder = LinkedActionsBuilder::new();
        f(&mut builder);

        let actions = builder.build();

        if !actions.is_empty() {
            self.action_queue(agent).push(
                config.order,
                (ActionType::Linked(actions, 0), config.repeat),
            );
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
        self.stop_current_action(agent, StopReason::Canceled);
        self.start_next_action(agent);
    }

    fn finish_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Finished);
        self.start_next_action(agent);
    }

    fn cancel_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
    }

    fn pause_action(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Paused);
    }

    fn skip_action(&mut self, agent: Entity) {
        if let Some((action, mut repeat)) = self.pop_next_action(agent) {
            if repeat.process() {
                self.action_queue(agent).push_back((action, repeat));
            }
        }
    }

    fn clear_actions(&mut self, agent: Entity) {
        self.stop_current_action(agent, StopReason::Canceled);
        self.action_queue(agent).clear();
    }
}

trait WorldHelperExt {
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason);
    fn start_next_action(&mut self, agent: Entity);
    fn take_current_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn pop_next_action(&mut self, agent: Entity) -> Option<ActionTuple>;
    fn action_queue(&mut self, agent: Entity) -> Mut<ActionQueue>;
    fn push_deferred_action(&mut self, command: impl Command);
    fn apply_deferred_actions(&mut self);
    fn has_current_action(&self, agent: Entity) -> bool;
}

impl WorldHelperExt for World {
    fn stop_current_action(&mut self, agent: Entity, reason: StopReason) {
        if let Some((mut current_action, mut repeat)) = self.take_current_action(agent) {
            self.get_mut::<ActionFinished>(agent)
                .unwrap()
                .bypass_change_detection()
                .reset_counts();

            match &mut current_action {
                ActionType::One(action) => {
                    action.on_stop(agent, self, reason);

                    match reason {
                        StopReason::Finished | StopReason::Canceled => {
                            if repeat.process() {
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Paused => {
                            self.action_queue(agent)
                                .push_front((current_action, repeat));
                        }
                    }
                }
                ActionType::Many(actions) => {
                    actions
                        .iter_mut()
                        .for_each(|action| action.on_stop(agent, self, reason));

                    match reason {
                        StopReason::Finished | StopReason::Canceled => {
                            if repeat.process() {
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Paused => {
                            self.action_queue(agent)
                                .push_front((current_action, repeat));
                        }
                    }
                }
                ActionType::Linked(actions, index) => {
                    match &mut actions[*index] {
                        OneOrMany::One(action) => action.on_stop(agent, self, reason),
                        OneOrMany::Many(actions) => actions
                            .iter_mut()
                            .for_each(|action| action.on_stop(agent, self, reason)),
                    }

                    match reason {
                        StopReason::Finished => {
                            *index += 1;

                            if *index < actions.len() {
                                self.action_queue(agent)
                                    .push_front((current_action, repeat));
                            } else if *index == actions.len() && repeat.process() {
                                *index = 0;
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Canceled => {
                            if repeat.process() {
                                *index = 0;
                                self.action_queue(agent).push_back((current_action, repeat));
                            }
                        }
                        StopReason::Paused => {
                            self.action_queue(agent)
                                .push_front((current_action, repeat));
                        }
                    }
                }
            }

            self.apply_deferred_actions();
        }
    }

    fn start_next_action(&mut self, agent: Entity) {
        if let Some((mut next_action, repeat)) = self.pop_next_action(agent) {
            match &mut next_action {
                ActionType::One(action) => action.on_start(agent, self),
                ActionType::Many(actions) => actions
                    .iter_mut()
                    .for_each(|action| action.on_start(agent, self)),
                ActionType::Linked(actions, index) => match &mut actions[*index] {
                    OneOrMany::One(action) => action.on_start(agent, self),
                    OneOrMany::Many(actions) => actions
                        .iter_mut()
                        .for_each(|action| action.on_start(agent, self)),
                },
            }

            self.get_mut::<CurrentAction>(agent).unwrap().0 = Some((next_action, repeat));

            self.apply_deferred_actions();
        }
    }

    fn take_current_action(&mut self, agent: Entity) -> Option<ActionTuple> {
        self.get_mut::<CurrentAction>(agent).unwrap().take()
    }

    fn pop_next_action(&mut self, agent: Entity) -> Option<ActionTuple> {
        self.action_queue(agent).pop_front()
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
