use bevy_ecs::prelude::*;

use crate::*;

pub trait ActionsWorldExt {
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig);
    fn stop_action(&mut self, actor: Entity);
    fn next_action(&mut self, actor: Entity);
    fn clear_actions(&mut self, actor: Entity);
    fn action_builder(&mut self, actor: Entity, config: AddConfig) -> ActionWorldBuilder;
}

impl ActionsWorldExt for World {
    fn add_action(&mut self, actor: Entity, action: impl IntoAction, config: AddConfig) {
        // Enqueue action
        let action_tuple = (action.into_boxed(), config.into());
        let mut actions = self.get_mut::<ActionQueue>(actor).unwrap();
        match config.order {
            AddOrder::Front => actions.0.push_front(action_tuple),
            AddOrder::Back => actions.0.push_back(action_tuple),
        }

        // Start next action if nothing is currently running
        if config.start {
            let has_current = self.get::<CurrentAction>(actor).unwrap().0.is_some();
            if !has_current {
                self.next_action(actor);
            }
        }
    }

    fn stop_action(&mut self, actor: Entity) {
        // Stop current action
        let current = self.get_mut::<CurrentAction>(actor).unwrap().0.take();
        if let Some((mut action, cfg)) = current {
            action.stop(actor, self);
            let mut actions = self.get_mut::<ActionQueue>(actor).unwrap();
            // Push stopped action to front so it runs again when next command is called
            actions.0.push_front((action, cfg));
        }
    }

    fn next_action(&mut self, actor: Entity) {
        // Remove current action
        let current = self.get_mut::<CurrentAction>(actor).unwrap().0.take();
        if let Some((mut action, cfg)) = current {
            action.remove(actor, self);
            if cfg.repeat {
                // Add action to back of queue again if repeat
                let mut actions = self.get_mut::<ActionQueue>(actor).unwrap();
                actions.0.push_back((action, cfg));
            }
        }

        // Get next action
        let next = self.get_mut::<ActionQueue>(actor).unwrap().0.pop_front();

        // Set next action
        let mut commands = ActionCommands::default();
        if let Some((mut action, cfg)) = next {
            action.add(actor, self, &mut commands);
            let mut current = self.get_mut::<CurrentAction>(actor).unwrap();
            current.0 = Some((action, cfg));
        }

        commands.apply(self);
    }

    fn clear_actions(&mut self, actor: Entity) {
        // Remove current
        let current = self.get_mut::<CurrentAction>(actor).unwrap().0.take();
        if let Some((mut action, _)) = current {
            action.remove(actor, self);
        }

        // Clear remaining
        let mut actions = self.get_mut::<ActionQueue>(actor).unwrap();
        actions.0.clear();
    }

    fn action_builder(&mut self, actor: Entity, config: AddConfig) -> ActionWorldBuilder {
        ActionWorldBuilder {
            actor,
            config,
            actions: Vec::default(),
            world: self,
        }
    }
}

pub struct ActionWorldBuilder<'a> {
    actor: Entity,
    config: AddConfig,
    actions: Vec<Box<dyn Action>>,
    world: &'a mut World,
}

impl<'a> ActionWorldBuilder<'a> {
    pub fn add(mut self, action: impl IntoAction) -> Self {
        self.actions.push(action.into_boxed());
        self
    }

    pub fn reverse(mut self) -> Self {
        self.actions.reverse();
        self
    }

    pub fn apply(self) {
        for action in self.actions {
            self.world.add_action(self.actor, action, self.config);
        }
    }
}
