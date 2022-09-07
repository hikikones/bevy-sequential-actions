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
        let action_tuple = (action.into_boxed(), cfg.into());
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
                let action = ActionType::Many(actions.collect::<Box<[_]>>());
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
        // self.stop_current_action(StopReason::Finished);
        // self.start_next_action();

        if let Some((_, cfg)) = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .0
            .as_mut()
        {
            cfg.finished += 1;
        }

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
        if let Some((action, cfg)) = self.pop_next_action() {
            self.handle_repeat(action, cfg);
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
        if let Some((mut action, cfg)) = self.take_current_action() {
            match &mut action {
                ActionType::Single(action) => {
                    action.on_stop(self.entity, self.world, reason);
                }
                ActionType::Many(actions) => {
                    for action in actions.iter_mut() {
                        action.on_stop(self.entity, self.world, reason);
                    }
                }
            }

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    self.handle_repeat(action, cfg);
                }
                StopReason::Paused => {
                    self.get_action_queue().push_front((action, cfg));
                }
            }
        }
    }

    fn start_next_action(&mut self) {
        if let Some((mut action, cfg)) = self.pop_next_action() {
            let mut commands = ActionCommands::default();

            match &mut action {
                ActionType::Single(action) => {
                    action.on_start(self.entity, self.world, &mut commands);
                }
                ActionType::Many(actions) => {
                    for action in actions.iter_mut() {
                        action.on_start(self.entity, self.world, &mut commands);
                    }
                }
            }

            if let Some(mut current) = self.world.get_mut::<CurrentAction>(self.entity) {
                **current = Some((action, cfg));
            }

            commands.apply(self.world);
        }
    }

    fn handle_repeat(&mut self, action: ActionType, mut cfg: ActionConfig) {
        match &mut cfg.repeat {
            Repeat::Amount(n) => {
                if *n > 0 {
                    *n -= 1;
                    self.get_action_queue().push_back((action, cfg));
                }
            }
            Repeat::Forever => {
                self.get_action_queue().push_back((action, cfg));
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
