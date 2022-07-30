use bevy_ecs::{prelude::*, system::CommandQueue};

use crate::*;

impl<'a> ActionsProxy<'a> for World {
    type Modifier = EntityWorldActions<'a>;

    fn actions(&'a mut self, entity: Entity) -> EntityWorldActions<'a> {
        EntityWorldActions {
            entity,
            config: AddConfig::default(),
            actions: Vec::new(),
            world: self,
        }
    }
}

/// Modify actions using [`World`].
pub struct EntityWorldActions<'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    world: &'a mut World,
}

impl ModifyActions for EntityWorldActions<'_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;

        self
    }

    fn add(mut self, action: impl IntoAction) -> Self {
        let action_tuple = (action.into_boxed(), self.config.into());
        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        match self.config.order {
            AddOrder::Front => actions.push_front(action_tuple),
            AddOrder::Back => actions.push_back(action_tuple),
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

    fn clear(mut self) -> Self {
        self.stop_current_action(StopReason::Canceled);

        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        actions.clear();

        self
    }

    fn push(mut self, action: impl IntoAction) -> Self {
        self.actions.push((action.into_boxed(), self.config));

        self
    }

    fn reverse(mut self) -> Self {
        self.actions.reverse();

        self
    }

    fn submit(mut self) -> Self {
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, self.world);

        for (action, config) in self.actions.drain(..) {
            commands.actions(self.entity).config(config).add(action);
        }

        command_queue.apply(self.world);

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
