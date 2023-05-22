use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_sequential_actions::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, many_agents);

fn many_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_agents");

    for agents in [10, 100, 1000, 10_000, 100_000, 1_000_000] {
        for system_kind in [
            QueueAdvancement::Normal,
            QueueAdvancement::Parallel,
            QueueAdvancement::Exclusive,
        ] {
            let mut bench = black_box(Benchmark::new(agents, system_kind));
            group.bench_function(format!("{agents} {system_kind:?}"), |b| {
                b.iter(|| bench.update());
            });
        }
    }

    group.finish();
}

#[derive(Deref, DerefMut)]
struct Benchmark(App);

impl Benchmark {
    fn new(agents: u32, system_kind: QueueAdvancement) -> Self {
        let mut app = App::new();
        app.add_plugin(SequentialActionsPlugin::<()>::new(
            system_kind,
            |app, system| {
                app.add_system(system);
            },
        ));

        for _ in 0..agents {
            let agent = app.world.spawn(ActionsBundle::new()).id();
            app.world.actions(agent).add(BenchAction);
        }

        Self(app)
    }
}

struct BenchAction;

impl Action for BenchAction {
    fn is_finished(&self, _agent: Entity, _world: &World) -> Finished {
        Finished(false)
    }
    fn on_start(&mut self, _agent: Entity, _world: &mut World) -> Finished {
        Finished(false)
    }
    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}
