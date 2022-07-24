use crate::*;

/// The trait that all actions must implement.
///
/// # Example
///
/// An empty action that does nothing.
/// All actions must declare when they are done.
/// This is done by calling [`next`](ModifyActionsExt::next) from either [`ActionCommands`] or [`Commands`].
///
/// ```rust
/// struct EmptyAction;
///
/// impl Action for EmptyAction {
///     fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands) {
///         // Action is finished, issue next.
///         commands.action(entity).next();
///     }
///
///     fn stop(&mut self, entity: Entity, world: &mut World) {}
/// }
/// ```
pub trait Action: Send + Sync {
    /// The method that is called when an [`action`](Action) is started.
    fn start(&mut self, entity: Entity, world: &mut World, commands: &mut ActionCommands);
    /// The method that is called when an [`action`](Action) is stopped.
    fn stop(&mut self, entity: Entity, world: &mut World);
}

/// Conversion into an [`Action`].
pub trait IntoAction {
    /// Convert `self` into `Box<dyn Action>`.
    fn into_boxed(self) -> Box<dyn Action>;
}

impl<T> IntoAction for T
where
    T: Action + 'static,
{
    fn into_boxed(self) -> Box<dyn Action> {
        Box::new(self)
    }
}

impl IntoAction for Box<dyn Action> {
    fn into_boxed(self) -> Box<dyn Action> {
        self
    }
}

pub trait Proxy<'a> {
    type Builder: ModifyActionsExt;
    fn action(&'a mut self, entity: Entity) -> Self::Builder;
}

pub struct Yoyo<'w, 's, 'a> {
    entity: Entity,
    config: AddConfig,
    actions: Vec<(Box<dyn Action>, AddConfig)>,
    commands: &'a mut Commands<'w, 's>,
}

impl<'w: 'a, 's: 'a, 'a> Proxy<'a> for Commands<'w, 's> {
    type Builder = Yoyo<'w, 's, 'a>;

    fn action(&'a mut self, entity: Entity) -> Yoyo<'w, 's, 'a> {
        Yoyo {
            entity,
            config: AddConfig::default(),
            actions: Vec::new(),
            commands: self,
        }
    }
}

impl<'w, 's, 'a> ModifyActionsExt for Yoyo<'w, 's, 'a> {
    fn config(self, config: AddConfig) -> Self {
        todo!()
    }

    fn add(self, action: impl IntoAction) -> Self {
        todo!()
    }

    fn next(self) -> Self {
        todo!()
    }

    fn stop(self) -> Self {
        todo!()
    }

    fn clear(self) -> Self {
        todo!()
    }

    fn push(self, action: impl IntoAction) -> Self {
        todo!()
    }

    fn reverse(self) -> Self {
        todo!()
    }

    fn submit(self) -> Self {
        todo!()
    }
}

// pub trait Proxy<'a> {
//     type Builder: ModifyActionsExt;
//     fn action(&mut self, entity: Entity) -> Self::Builder;
// }

// pub struct Yoyo<'w, 's, 'a> {
//     entity: Entity,
//     config: AddConfig,
//     actions: Vec<(Box<dyn Action>, AddConfig)>,
//     commands: &'a mut Commands<'w, 's>,
// }

// impl<'w: 'a, 's: 'a, 'a> Proxy<'a> for Commands<'w, 's> {
//     type Builder = Yoyo<'w, 's, 'a>;

//     fn action(&mut self, entity: Entity) -> Yoyo<'w, 's, 'a> {
//         Yoyo {
//             entity,
//             config: AddConfig::default(),
//             actions: Vec::new(),
//             commands: self,
//         }
//     }
// }

// impl<'w, 's, 'a> ModifyActionsExt for Yoyo<'w, 's, 'a> {
//     fn config(self, config: AddConfig) -> Self {
//         todo!()
//     }

//     fn add(self, action: impl IntoAction) -> Self {
//         todo!()
//     }

//     fn next(self) -> Self {
//         todo!()
//     }

//     fn stop(self) -> Self {
//         todo!()
//     }

//     fn clear(self) -> Self {
//         todo!()
//     }

//     fn push(self, action: impl IntoAction) -> Self {
//         todo!()
//     }

//     fn reverse(self) -> Self {
//         todo!()
//     }

//     fn submit(self) -> Self {
//         todo!()
//     }
// }

/// Extension methods for modifying actions.
pub trait ModifyActionsExt {
    /// Sets the current [`config`](AddConfig) for actions to be added.
    fn config(self, config: AddConfig) -> Self;

    /// Adds an [`action`](Action) to the queue with the current [`config`](AddConfig).
    fn add(self, action: impl IntoAction) -> Self;

    /// Starts the next [`action`](Action) in the queue by [`stopping`](Action::stop) the currently running action,
    /// and [`starting`](Action::start) the next action in the queue list.
    fn next(self) -> Self;

    /// [`Stops`](Action::stop) the currently running [`action`](Action) without removing it from the queue.
    fn stop(self) -> Self;

    /// [`Stops`](Action::stop) the currently running [`action`](Action), and clears any remaining.
    fn clear(self) -> Self;

    /// Pushes an [`action`](Action) to a list with the current [`config`](AddConfig).
    /// Pushed actions __will not__ be added to the queue until [`submit`](Self::submit) is called.
    fn push(self, action: impl IntoAction) -> Self;

    /// Reverses the order of the [`pushed`](Self::push) actions.
    fn reverse(self) -> Self;

    /// Submit the [`pushed`](Self::push) actions by draining the list and adding them to the queue.
    fn submit(self) -> Self;
}
