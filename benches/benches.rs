use std::time::Duration;

use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};

use bevy_sequential_actions::*;

criterion_main!(benches);
criterion_group!(benches, many_agents);

fn many_agents(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_agents");
    // group.warm_up_time(Duration::from_millis(500));
    // group.measurement_time(Duration::from_secs(2));
    // group.sampling_mode(SamplingMode::Flat);
    // group.sample_size(10);

    for agents in [10, 100, 1000, 10_000, 100_000, 1_000_000] {
        let mut bench = black_box(Benchmark::new(agents, false));
        group.bench_function(format!("{agents}"), |b| {
            b.iter(|| bench.run());
        });

        let mut bench = black_box(Benchmark::new(agents, true));
        group.bench_function(format!("{agents} (parallel)"), |b| {
            b.iter(|| bench.run());
        });

        // group.sample_size(sample_size);
        // let mut app = black_box(Benchmark::new(max, CheckActionsExec::Seq));
        // group.bench_function(format!("many_countdowns_{max}"), |b| {
        //     b.iter(|| app.run());
        // });

        // let mut app = black_box(Benchmark::new(max, CheckActionsExec::Parallel));
        // group.bench_function(format!("many_countdowns_par_{max}"), |b| {
        //     b.iter(|| app.run());
        // });

        // group.bench_function(format!("many_countdowns_{max}"), |bencher| {
        //     bencher.iter(|| black_box(run_many_countdowns(max, CheckActionsExec::Seq)));
        // });
        // group.bench_function(format!("many_countdowns_par_{max}"), |bencher| {
        //     bencher.iter(|| black_box(run_many_countdowns(max, CheckActionsExec::Parallel)));
        // });
    }

    // group.bench_function("many_countdowns_10", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(10, CheckActionsExec::Seq)));
    // });
    // group.bench_function("many_countdowns_par_10", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(10, CheckActionsExec::Parallel)));
    // });

    // group.bench_function("many_countdowns_100", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(100, CheckActionsExec::Seq)));
    // });
    // group.bench_function("many_countdowns_par_100", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(100, CheckActionsExec::Parallel)));
    // });

    // group.bench_function("many_countdowns_1000", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(1000, CheckActionsExec::Seq)));
    // });
    // group.bench_function("many_countdowns_par_1000", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(1000, CheckActionsExec::Parallel)));
    // });

    // group.bench_function("many_countdowns_10000", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(10000, CheckActionsExec::Seq)));
    // });
    // group.bench_function("many_countdowns_par_10000", |bencher| {
    //     bencher.iter(|| black_box(run_many_countdowns(10000, CheckActionsExec::Parallel)));
    // });

    group.finish();
}

struct Benchmark {
    schedule: Schedule,
    world: World,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
struct Update;

impl Benchmark {
    fn new(agents: i32, parallel: bool) -> Self {
        let mut schedule = Schedule::new();

        if parallel {
            schedule
                .add_systems(SequentialActionsPlugin::<DefaultAgentMarker>::get_parallel_systems());
        } else {
            schedule.add_systems(SequentialActionsPlugin::<DefaultAgentMarker>::get_systems());
        }

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
    fn on_start(&mut self, _agent: Entity, _world: &mut World) {}
    fn on_stop(&mut self, _agent: Entity, _world: &mut World, _reason: StopReason) {}
}

// struct Benchmark {
//     app: App,
//     // max: i32,
//     // exec: CheckActionsExec,
// }

// impl Benchmark {
//     fn new(max: i32, exec: CheckActionsExec) -> Self {
//         let mut app = App::new();
//         app.add_startup_system(move |mut commands: Commands| {
//             for i in 0..max {
//                 let agent = commands.spawn(ActionsBundle::default()).id();
//                 commands.actions(agent).add(CountdownAction::new(i));
//             }
//         })
//         .add_systems(SequentialActionsPlugin::<DefaultAgentMarker>::get_systems(
//             exec,
//         ))
//         .add_system(countdown);

//         Self { app }
//     }

//     fn run(&mut self) {
//         self.app.update();
//     }
// }

// fn run_many_countdowns(max: i32, exec: ActionQueueCommandsType) {
//     let mut app = App::new();
//     app.add_startup_system(move |mut commands: Commands| {
//         for i in 0..max {
//             let agent = commands.spawn(ActionsBundle::default()).id();
//             commands.actions(agent).add(CountdownAction::new(i));
//         }
//     })
//     .add_systems(SequentialActionsPlugin::<DefaultAgentMarker>::get_systems(
//         exec,
//     ))
//     .add_system(countdown);

//     for _ in 0..max.min(10) {
//         app.update();
//     }
// }

struct CountdownAction {
    count: i32,
    current: Option<i32>,
}

impl CountdownAction {
    const fn new(count: i32) -> Self {
        Self {
            count,
            current: None,
        }
    }
}

impl Action for CountdownAction {
    fn is_finished(&self, agent: Entity, world: &World) -> bool {
        world.get::<Countdown>(agent).unwrap().0 <= 0
    }

    fn on_start(&mut self, agent: Entity, world: &mut World) {
        let count = self.current.unwrap_or(self.count);
        world.entity_mut(agent).insert(Countdown(count));
    }

    fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
        let countdown = world.entity_mut(agent).take::<Countdown>();
        if let StopReason::Paused = reason {
            self.current = Some(countdown.unwrap().0);
        }
    }
}

#[derive(Component)]
struct Countdown(i32);

fn countdown(mut countdown_q: Query<&mut Countdown>) {
    for mut countdown in &mut countdown_q {
        countdown.0 -= 1;
    }
}
