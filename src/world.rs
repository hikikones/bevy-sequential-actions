use bevy_ecs::prelude::*;

use crate::*;

impl<'a> ActionsProxy<'a> for World {
    type Modifier = EntityWorldActions<'a>;

    fn actions(&'a mut self, entity: Entity) -> EntityWorldActions<'a> {
        EntityWorldActions {
            entity,
            config: AddConfig::default(),
            world: self,
        }
    }
}

/// Modify actions using [`World`].
pub struct EntityWorldActions<'w> {
    entity: Entity,
    config: AddConfig,
    world: &'w mut World,
}

impl ModifyActions for EntityWorldActions<'_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add<T>(self, action: T) -> Self
    where
        T: IntoBoxedAction,
    {
        self.world.add_action(self.entity, self.config, action);
        self
    }

    fn add_many<T>(self, mode: ExecutionMode, actions: T) -> Self
    where
        T: BoxedActionIter,
    {
        self.world
            .add_actions(self.entity, self.config, mode, actions);
        self
    }

    fn next(self) -> Self {
        self.world.next_action(self.entity);
        self
    }

    fn cancel(self) -> Self {
        self.world.cancel_action(self.entity);
        self
    }

    fn pause(self) -> Self {
        self.world.pause_action(self.entity);
        self
    }

    fn skip(self) -> Self {
        self.world.skip_action(self.entity);
        self
    }

    fn clear(self) -> Self {
        self.world.clear_actions(self.entity);
        self
    }
}

pub(super) trait WorldActionsExt {
    fn add_action(&mut self, entity: Entity, config: AddConfig, action: impl IntoBoxedAction);
    fn add_actions(
        &mut self,
        entity: Entity,
        config: AddConfig,
        mode: ExecutionMode,
        actions: impl BoxedActionIter,
    );
    fn next_action(&mut self, entity: Entity);
    fn finish_action(&mut self, entity: Entity);
    fn cancel_action(&mut self, entity: Entity);
    fn pause_action(&mut self, entity: Entity);
    fn skip_action(&mut self, entity: Entity);
    fn clear_actions(&mut self, entity: Entity);
    fn stop_current_action(&mut self, entity: Entity, reason: StopReason);
    fn handle_repeat_action(&mut self, entity: Entity, action: ActionType, state: ActionState);
    fn start_next_action(&mut self, entity: Entity);
    fn take_current_action(&mut self, entity: Entity) -> Option<ActionTuple>;
    fn pop_next_action(&mut self, entity: Entity) -> Option<ActionTuple>;
    fn has_current_action(&self, entity: Entity) -> bool;
    fn action_queue<'a>(&'a mut self, entity: Entity) -> Mut<'a, ActionQueue>;
}

impl WorldActionsExt for World {
    fn add_action(&mut self, entity: Entity, config: AddConfig, action: impl IntoBoxedAction) {
        let action_tuple = (ActionType::Single(action.into_boxed()), config.into());
        let mut queue = self.action_queue(entity);

        match config.order {
            AddOrder::Back => queue.push_back(action_tuple),
            AddOrder::Front => queue.push_front(action_tuple),
        }

        if config.start && !self.has_current_action(entity) {
            self.start_next_action(entity);
        }
    }

    fn add_actions(
        &mut self,
        entity: Entity,
        config: AddConfig,
        mode: ExecutionMode,
        actions: impl BoxedActionIter,
    ) {
        let mut queue = self.action_queue(entity);

        match mode {
            ExecutionMode::Sequential => match config.order {
                AddOrder::Back => {
                    for action in actions {
                        queue.push_back((ActionType::Single(action), config.into()));
                    }
                }
                AddOrder::Front => {
                    for action in actions.rev() {
                        queue.push_front((ActionType::Single(action), config.into()));
                    }
                }
            },
            ExecutionMode::Parallel => {
                let action = ActionType::Multiple(actions.collect::<Box<[_]>>());
                match config.order {
                    AddOrder::Back => queue.push_back((action, config.into())),
                    AddOrder::Front => queue.push_front((action, config.into())),
                }
            }
        }

        if config.start && !self.has_current_action(entity) {
            self.start_next_action(entity);
        }
    }

    fn next_action(&mut self, entity: Entity) {
        self.stop_current_action(entity, StopReason::Canceled);
        self.start_next_action(entity);
    }

    fn finish_action(&mut self, entity: Entity) {
        self.stop_current_action(entity, StopReason::Finished);
        self.start_next_action(entity);
    }

    fn cancel_action(&mut self, entity: Entity) {
        self.stop_current_action(entity, StopReason::Canceled);
        self.start_next_action(entity);
    }

    fn pause_action(&mut self, entity: Entity) {
        self.stop_current_action(entity, StopReason::Paused);
    }

    fn skip_action(&mut self, entity: Entity) {
        if let Some((action, state)) = self.pop_next_action(entity) {
            self.handle_repeat_action(entity, action, state);
        }
    }

    fn clear_actions(&mut self, entity: Entity) {
        self.stop_current_action(entity, StopReason::Canceled);
        self.action_queue(entity).clear();
    }

    fn stop_current_action(&mut self, entity: Entity, reason: StopReason) {
        if let Some((mut action, state)) = self.take_current_action(entity) {
            match &mut action {
                ActionType::Single(action) => {
                    action.on_stop(entity, self, reason);
                }
                ActionType::Multiple(actions) => {
                    for action in actions.iter_mut() {
                        action.on_stop(entity, self, reason);
                    }
                }
            }

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    self.handle_repeat_action(entity, action, state);
                }
                StopReason::Paused => {
                    self.action_queue(entity).push_front((action, state));
                }
            }

            let mut finished = self.get_mut::<FinishedCount>(entity).unwrap();
            if finished.0 != 0 {
                finished.0 = 0;
            }
        }
    }

    fn handle_repeat_action(&mut self, entity: Entity, action: ActionType, mut state: ActionState) {
        match &mut state.repeat {
            Repeat::Amount(n) => {
                if *n > 0 {
                    *n -= 1;
                    self.action_queue(entity).push_back((action, state));
                }
            }
            Repeat::Forever => {
                self.action_queue(entity).push_back((action, state));
            }
        }
    }

    fn start_next_action(&mut self, entity: Entity) {
        if let Some((mut action, state)) = self.pop_next_action(entity) {
            let mut commands = ActionCommands::default();

            match &mut action {
                ActionType::Single(action) => {
                    action.on_start(entity, self, &mut commands);
                }
                ActionType::Multiple(actions) => {
                    for action in actions.iter_mut() {
                        action.on_start(entity, self, &mut commands);
                    }
                }
            }

            if let Some(mut current) = self.get_mut::<CurrentAction>(entity) {
                **current = Some((action, state));
            }

            commands.apply(self);
        }
    }

    fn take_current_action(&mut self, entity: Entity) -> Option<ActionTuple> {
        self.get_mut::<CurrentAction>(entity).unwrap().take()
    }

    fn pop_next_action(&mut self, entity: Entity) -> Option<ActionTuple> {
        self.action_queue(entity).pop_front()
    }

    fn has_current_action(&self, entity: Entity) -> bool {
        self.get::<CurrentAction>(entity).unwrap().is_some()
    }

    fn action_queue<'a>(&'a mut self, entity: Entity) -> Mut<'a, ActionQueue> {
        self.get_mut::<ActionQueue>(entity).unwrap()
    }
}
