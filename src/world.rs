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
        // Enqueue action
        let action_tuple = (action.into_boxed(), self.config.into());
        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        match self.config.order {
            AddOrder::Front => actions.push_front(action_tuple),
            AddOrder::Back => actions.push_back(action_tuple),
        }

        // Start next action if nothing is currently running
        if self.config.start && !self.has_current_action() {
            self.next_action();
        }

        self
    }

    fn finish(mut self) -> Self {
        self.remove_current_action(true);
        self.next_action();
        self
    }

    fn cancel(mut self) -> Self {
        self.remove_current_action(false);
        self.next_action();
        self
    }

    fn pause(mut self) -> Self {
        let current = self.take_current_action();

        // Pause current action
        if let Some((mut action, mut cfg)) = current {
            action.pause(self.entity, self.world);

            // Put action back into queue with is_paused enabled
            cfg.is_paused = true;
            let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
            actions.push_back((action, cfg));
        }

        self
    }

    fn resume(mut self) -> Self {
        self.next_action();
        self
    }

    fn clear(mut self) -> Self {
        let current = self.take_current_action();

        // Cancel current action
        if let Some((mut action, _)) = current {
            action.cancel(self.entity, self.world);
        }

        // Clear remaining
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
    fn remove_current_action(&mut self, success: bool) {
        let current = self.take_current_action();

        // Finish or cancel current action
        if let Some((mut action, cfg)) = current {
            if success {
                action.finish(self.entity, self.world);
            } else {
                action.cancel(self.entity, self.world);
            }

            if cfg.repeat {
                // Add action back to queue again if repeat
                let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
                actions.push_back((action, cfg));
            }
        }
    }

    fn next_action(&mut self) {
        let next = self.pop_next_action();

        // Start or resume and set current action
        if let Some((mut action, mut cfg)) = next {
            let mut commands = ActionCommands::default();

            if cfg.is_paused {
                cfg.is_paused = false;
                action.resume(self.entity, self.world, &mut commands);
            } else {
                action.start(self.entity, self.world, &mut commands);
            }

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
