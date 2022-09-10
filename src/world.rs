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

    fn add<T>(mut self, action: T) -> Self
    where
        T: IntoBoxedAction,
    {
        let cfg = self.config;
        let action_tuple = (ActionType::Single(action.into_boxed()), cfg.into());
        let mut queue = self.get_action_queue();

        match cfg.order {
            AddOrder::Back => queue.push_back(action_tuple),
            AddOrder::Front => queue.push_front(action_tuple),
        }

        if cfg.start && !self.has_current_action() {
            self.start_next_action();
        }

        self
    }

    fn add_many<T>(mut self, mode: ExecutionMode, actions: T) -> Self
    where
        T: BoxedActionIter,
    {
        let cfg = self.config;
        let mut queue = self.get_action_queue();

        match mode {
            ExecutionMode::Sequential => match cfg.order {
                AddOrder::Back => {
                    for action in actions {
                        queue.push_back((ActionType::Single(action), cfg.into()));
                    }
                }
                AddOrder::Front => {
                    for action in actions.rev() {
                        queue.push_front((ActionType::Single(action), cfg.into()));
                    }
                }
            },
            ExecutionMode::Parallel => {
                let action = ActionType::Multiple(actions.collect::<Box<[_]>>());
                match cfg.order {
                    AddOrder::Back => queue.push_back((action, cfg.into())),
                    AddOrder::Front => queue.push_front((action, cfg.into())),
                }
            }
        }

        if cfg.start && !self.has_current_action() {
            self.start_next_action();
        }

        self
    }

    fn next(mut self) -> Self {
        self.stop_current_action(StopReason::Canceled);
        self.start_next_action();
        self
    }

    fn finish(mut self) -> Self {
        self.stop_current_action(StopReason::Finished);
        self.start_next_action();
        self
    }

    fn pause(mut self) -> Self {
        self.stop_current_action(StopReason::Paused);
        self
    }

    fn stop(mut self, reason: StopReason) -> Self {
        self.stop_current_action(reason);
        self
    }

    fn skip(mut self) -> Self {
        if let Some((action, state)) = self.pop_next_action() {
            self.handle_repeat(action, state);
        }
        self
    }

    fn clear(mut self) -> Self {
        self.stop_current_action(StopReason::Canceled);
        self.get_action_queue().clear();
        self
    }
}

impl EntityWorldActions<'_> {
    fn stop_current_action(&mut self, reason: StopReason) {
        if let Some((mut action_type, state)) = self.take_current_action() {
            match &mut action_type {
                ActionType::Single(action) => {
                    action.on_stop(self.entity, self.world, reason);
                }
                ActionType::Multiple(actions) => {
                    for action in actions.iter_mut() {
                        action.on_stop(self.entity, self.world, reason);
                    }
                }
            }

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    self.handle_repeat(action_type, state);
                }
                StopReason::Paused => {
                    self.get_action_queue().push_front((action_type, state));
                }
            }

            let mut finished = self.world.get_mut::<FinishedCount>(self.entity).unwrap();
            if finished.0 != 0 {
                finished.0 = 0;
            }
        }
    }

    fn start_next_action(&mut self) {
        if let Some((mut action_type, state)) = self.pop_next_action() {
            let mut commands = ActionCommands::default();

            match &mut action_type {
                ActionType::Single(action) => {
                    action.on_start(self.entity, self.world, &mut commands);
                }
                ActionType::Multiple(actions) => {
                    for action in actions.iter_mut() {
                        action.on_start(self.entity, self.world, &mut commands);
                    }
                }
            }

            if let Some(mut current) = self.world.get_mut::<CurrentAction>(self.entity) {
                **current = Some((action_type, state));
            }

            commands.apply(self.world);
        }
    }

    fn handle_repeat(&mut self, action: ActionType, mut state: ActionState) {
        match &mut state.repeat {
            Repeat::Amount(n) => {
                if *n > 0 {
                    *n -= 1;
                    self.get_action_queue().push_back((action, state));
                }
            }
            Repeat::Forever => {
                self.get_action_queue().push_back((action, state));
            }
        }
    }

    fn take_current_action(&mut self) -> Option<ActionTuple> {
        self.world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .take()
    }

    fn pop_next_action(&mut self) -> Option<ActionTuple> {
        self.world
            .get_mut::<ActionQueue>(self.entity)
            .unwrap()
            .pop_front()
    }

    fn get_action_queue(&mut self) -> Mut<ActionQueue> {
        self.world.get_mut::<ActionQueue>(self.entity).unwrap()
    }

    fn has_current_action(&self) -> bool {
        self.world
            .get::<CurrentAction>(self.entity)
            .unwrap()
            .is_some()
    }
}

pub(super) trait WorldExt {
    fn add_action(&mut self, entity: Entity, config: AddConfig, action: impl IntoBoxedAction);
    fn add_actions(
        &mut self,
        entity: Entity,
        mode: ExecutionMode,
        config: AddConfig,
        actions: impl BoxedActionIter,
    );
    fn next_action(&mut self, entity: Entity);
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

impl WorldExt for World {
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
        mode: ExecutionMode,
        config: AddConfig,
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

    fn pause_action(&mut self, entity: Entity) {
        self.stop_current_action(entity, StopReason::Paused);
        self.start_next_action(entity);
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
