use bevy_ecs::{prelude::*, system::CommandQueue};

use crate::*;

/// Extension method on [`World`] for modifying actions.
pub trait EntityWorldActionsExt {
    /// Returns an [`EntityWorldActions`] for the requested [`Entity`].
    ///
    /// ## Warning
    ///
    /// Do not modify actions using [`World`] inside the implementation of an [`Action`].
    /// Actions need to be properly queued, which is what [`ActionCommands`] does.
    /// ```rust
    /// struct EmptyAction;
    ///
    /// impl Action for EmptyAction {
    ///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
    ///         // Bad
    ///         world.action(entity).next();
    ///
    ///         // Good
    ///         commands.action(entity).next();
    ///     }
    ///
    ///     fn stop(&mut self, entity: Entity, world: &mut World) {}
    /// }
    ///```
    fn action(&mut self, entity: Entity) -> EntityWorldActions;
}

impl EntityWorldActionsExt for World {
    fn action(&mut self, entity: Entity) -> EntityWorldActions {
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

impl ModifyActionsExt for EntityWorldActions<'_> {
    fn config(mut self, config: AddConfig) -> Self {
        self.config = config;
        self
    }

    fn add(self, action: impl IntoAction) -> Self {
        // Enqueue action
        let action_tuple = (action.into_boxed(), self.config.into());
        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        match self.config.order {
            AddOrder::Front => actions.0.push_front(action_tuple),
            AddOrder::Back => actions.0.push_back(action_tuple),
        }

        // Start next action if nothing is currently running
        if self.config.start {
            let has_current = self
                .world
                .get::<CurrentAction>(self.entity)
                .unwrap()
                .0
                .is_some();

            if !has_current {
                return self.next();
            }
        }

        self
    }

    fn next(self) -> Self {
        // Get current action
        let current = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .0
            .take();

        // Remove current action
        if let Some((mut action, cfg)) = current {
            action.stop(self.entity, self.world);
            if cfg.repeat {
                // Add action to back of queue again if repeat
                let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
                actions.0.push_back((action, cfg));
            }
        }

        // Get next action
        let next = self
            .world
            .get_mut::<ActionQueue>(self.entity)
            .unwrap()
            .0
            .pop_front();

        // Set next action
        let mut commands = ActionCommands::default();
        if let Some((mut action, cfg)) = next {
            action.start(self.entity, self.world, &mut commands);
            if let Some(mut current) = self.world.get_mut::<CurrentAction>(self.entity) {
                current.0 = Some((action, cfg));
            }
        }

        commands.apply(self.world);

        self
    }

    fn stop(self) -> Self {
        // Get current action
        let current = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .0
            .take();

        // Remove current action
        if let Some((mut action, cfg)) = current {
            action.stop(self.entity, self.world);
            let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
            // Push stopped action to front so it runs again when next action is called
            actions.0.push_front((action, cfg));
        }

        self
    }

    fn clear(self) -> Self {
        // Get current action
        let current = self
            .world
            .get_mut::<CurrentAction>(self.entity)
            .unwrap()
            .0
            .take();

        // Remove current action
        if let Some((mut action, _)) = current {
            action.stop(self.entity, self.world);
        }

        // Clear remaining
        let mut actions = self.world.get_mut::<ActionQueue>(self.entity).unwrap();
        actions.0.clear();

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
            commands.action(self.entity).config(config).add(action);
        }

        command_queue.apply(self.world);

        self
    }
}
