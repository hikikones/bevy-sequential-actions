use bevy_ecs::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bevy_sequential_actions::*;

criterion_main!(benches);
criterion_group!(benches, many_agents);

fn many_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_agents");

    for agents in [10, 100, 1000, 10_000, 100_000, 1_000_000] {
        let mut bench = black_box(Benchmark::new(agents, false));
        group.bench_function(format!("{agents}"), |b| {
            b.iter(|| bench.run());
        });

        let mut bench = black_box(Benchmark::new(agents, true));
        group.bench_function(format!("{agents} (parallel)"), |b| {
            b.iter(|| bench.run());
        });
    }

    group.finish();
}

struct Benchmark {
    schedule: Schedule,
    world: World,
}

impl Benchmark {
    fn new(agents: u32, parallel: bool) -> Self {
        let mut schedule = Schedule::new();
        schedule.add_systems(if parallel {
            SequentialActionsPlugin::<DefaultAgentMarker>::get_parallel_systems()
        } else {
            SequentialActionsPlugin::<DefaultAgentMarker>::get_systems()
        });

        let mut world = World::new();
        for _ in 0..agents {
            let agent = world.spawn(ActionsBundle::default()).id();
            world.actions(agent).add(BenchAction);
        }

        Self { schedule, world }
    }

    fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
}

struct BenchAction;

impl Action for BenchAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> bool {
        false
    }
    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> bool {
        false
    }
    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
