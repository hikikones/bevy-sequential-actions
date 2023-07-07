use std::time::Duration;

use bevy_app::{prelude::*, AppExit, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use bevy_ecs::prelude::*;

use bevy_sequential_actions::*;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 10.0,
        )))
        .add_plugin(ScheduleRunnerPlugin)
        .add_startup_system(setup)
        // Add custom system for action queue advancement
        .add_system(check_actions_exclusive)
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
    mut agent_q: Local<QueryState<(Entity, &CurrentAction, &Id)>>,
) {
    // Collect all agents with finished action
    let mut finished_agents = agent_q
        .iter(world)
        .filter(|&(agent, current_action, _)| {
            current_action
                .as_ref()
                .map(|action| action.is_finished(agent, world))
                .unwrap_or(false)
        })
        .map(|(agent, _, id)| (agent, id.0))
        .collect::<Vec<_>>();

    // Sort by id in reverse
    finished_agents.sort_by_key(|&(_, id)| std::cmp::Reverse(id));

    // Advance the action queue
    finished_agents.into_iter().for_each(|(agent, _)| {
        ActionHandler::stop_current(agent, StopReason::Finished, world);
        ActionHandler::start_next(agent, world);
    });
}

struct PrintIdAction;

impl Action for PrintIdAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        true
    }

    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        false
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, _reason: StopReason) {
        let id = world.get::<Id>(agent).unwrap().0;

        // Observe that id is printed in descending order
        println!("Agent: {agent:?}, Id: {id}");

        if id == 0 {
            world.send_event(AppExit);
        }
    }
}
