use std::time::Duration;

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_systems(Startup, setup)
        // Add custom system for action queue advancement
        .add_systems(Last, check_actions_exclusive)
        .run();
}

fn setup(mut commands: Commands) {
    for i in 0..10 {
        // Spawn agents with id in ascending order
        let agent = commands.spawn((ActionsBundle::new(), Id(i))).id();
        commands.actions(agent).add(PrintIdAction);
    }
}

#[derive(Component)]
struct Id(u32);

fn check_actions_exclusive(
    world: &mut World,
    mut finished: Local<Vec<(Entity, u32)>>,
    mut agent_q: Local<QueryState<(Entity, &CurrentAction, &Id)>>,
) {
    // Collect all agents with finished action
    finished.extend(
        agent_q
            .iter(world)
            .filter(|&(agent, current_action, _)| {
                current_action
                    .as_ref()
                    .map(|action| action.is_finished(agent, world))
                    .unwrap_or(false)
            })
            .map(|(agent, _, id)| (agent, id.0)),
    );

    // Sort by id in reverse
    finished.sort_by_key(|&(_, id)| std::cmp::Reverse(id));

    // Advance the action queue
    for (agent, _) in finished.drain(..) {
        SequentialActionsPlugin::stop_current_action(agent, StopReason::Finished, world);
        SequentialActionsPlugin::start_next_action(agent, world);
    }
}

struct PrintIdAction;

impl Action for PrintIdAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        false
    }

    fn on_stop(&mut self, agent: Option<Entity>, world: &mut World, _reason: StopReason) {
        let agent = agent.unwrap();
        let id = world.get::<Id>(agent).unwrap().0;

        // Observe that id is printed in descending order
        println!("Agent: {agent}, Id: {id}");

        if id == 0 {
            world.send_event(AppExit::Success);
        }
    }
}
