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
        T: IntoAction,
    {
        let mut queue = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        let action_tuple = (action.into_boxed(), self.config.into());

        match self.config.order {
            AddOrder::Back => queue.push_back(action_tuple),
            AddOrder::Front => queue.push_front(action_tuple),
        }

        if self.config.start && !self.has_current_action() {
            self.start_next_action();
        }

        self
    }

    fn add_many<T>(mut self, actions: T) -> Self
    where
        T: BoxedActionIter,
    {
        let mut queue = self.world.get_mut::<ActionQueue>(self.entity).unwrap();

        match self.config.order {
            AddOrder::Back => {
                for action in actions {
                    queue.push_back((action, self.config.into()));
                }
            }
            AddOrder::Front => {
                for action in actions.rev() {
                    queue.push_front((action, self.config.into()));
                }
            }
        }

        if self.config.start && !self.has_current_action() {
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
        if let Some((action, cfg)) = self.pop_next_action() {
            if cfg.repeat {
                let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
                actions.push_back((action, cfg));
            }
        }

        self
    }

    fn clear(mut self) -> Self {
        self.stop_current_action(StopReason::Canceled);

        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        actions.clear();

        self
    }
}

impl EntityWorldActions<'_> {
    fn stop_current_action(&mut self, reason: StopReason) {
        if let Some((mut action, cfg)) = self.take_current_action() {
            action.on_stop(self.entity, self.world, reason);

            match reason {
                StopReason::Finished | StopReason::Canceled => {
                    if cfg.repeat {
                        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
                        actions.push_back((action, cfg));
                    }
                }
                StopReason::Paused => {
                    let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
                    actions.push_front((action, cfg));
                }
            }
        }
    }

    fn start_next_action(&mut self) {
        if let Some((mut action, cfg)) = self.pop_next_action() {
            let mut commands = ActionCommands::default();
            action.on_start(self.entity, self.world, &mut commands);

            if let Some(mut current) = self.world.get_mut::<CurrentAction>(self.entity) {
                **current = Some((action, cfg));
            }

            commands.apply(self.world);
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

    fn has_current_action(&self) -> bool {
        self.world
            .get::<CurrentAction>(self.entity)
            .unwrap()
            .is_some()
    }
}

// pub(super) trait WorldHelperExt {
//     fn add_action<T>(&mut self, entity: Entity, action: T, config: AddConfig)
//     where
//         T: IntoAction;
//     fn add_many_actions<T>(&mut self, entity: Entity, actions: T)
//     where
//         T: DoubleEndedIterator<Item = BoxedAction>;
//     fn stop_current_action(&mut self, entity: Entity, reason: StopReason);
//     fn start_next_action(&mut self, entity: Entity);
//     fn take_current_action(&mut self, entity: Entity) -> Option<ActionTuple>;
//     fn pop_next_action(&mut self, entity: Entity) -> Option<ActionTuple>;
//     fn has_current_action(&self, entity: Entity) -> bool;
// }
