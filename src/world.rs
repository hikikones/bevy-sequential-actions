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

    fn add(self, action: impl IntoAction) -> Self {
        // Enqueue action
        let action_tuple = (action.into_boxed(), self.config.into());
        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        match self.config.order {
            AddOrder::Front => actions.push_front(action_tuple),
            AddOrder::Back => actions.push_back(action_tuple),
        }

        // Start next action if nothing is currently running
        if self.config.start
            && self
                .world
                .get::<CurrentAction>(self.entity)
                .unwrap()
                .is_none()
        {
            return self.next();
        }

        self
    }

    fn next(self) -> Self {
        // Get current action
        let current = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .take();

        // Stop current action
        if let Some((mut action, cfg)) = current {
            action.stop(self.entity, self.world);
            if cfg.repeat {
                // Add action back to queue again if repeat
                let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
                actions.push_back((action, cfg));
            }
        }

        // Get next action
        let next = self
            .world
            .get_mut::<ActionQueue>(self.entity)
            .unwrap()
            .pop_front();

        // Start and set current action
        if let Some((mut action, cfg)) = next {
            let mut commands = ActionCommands::default();
            action.start(self.entity, self.world, &mut commands);
            if let Some(mut current) = self.world.get_mut::<CurrentAction>(self.entity) {
                **current = Some((action, cfg));
            }
            commands.apply(self.world);
        }

        self
    }

    fn stop(self) -> Self {
        // Get current action
        let current = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .take();

        // Stop current action
        if let Some((mut action, cfg)) = current {
            action.stop(self.entity, self.world);
            let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
            // Push stopped action to front so it runs again when next action is called
            actions.push_front((action, cfg));
        }

        self
    }

    fn clear(self) -> Self {
        // Get current action
        let current = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .take();

        // Stop current action
        if let Some((mut action, _)) = current {
            action.stop(self.entity, self.world);
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
