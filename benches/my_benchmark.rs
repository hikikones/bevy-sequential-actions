use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bevy_sequential_actions::*;

// fn fibonacci(n: u64) -> u64 {
//     match n {
//         0 => 1,
//         1 => 1,
//         n => fibonacci(n - 1) + fibonacci(n - 2),
//     }
// }

// fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);

criterion_group!(benches, my_bench);
criterion_main!(benches);

fn my_bench(c: &mut Criterion) {
    let mut app = App::new();
    app.add_plugin(SequentialActionsPlugin::default())
        .add_startup_system(setup)
        .add_system(countdown);

    // for i in 0..1000 {
    //     let agent = app.world.spawn(ActionsBundle::default()).id();
    //     app.world.actions(agent).add(CountdownAction::new(i));
    // }

    let mut group = c.benchmark_group("many_countdowns");
    group.significance_level(0.1).sample_size(1000);
    group.bench_function("update", |b| b.iter(|| black_box(app.update())));
    group.finish();

    // c.bench_function("countdown", |b| b.iter(|| black_box(app.update())));
}

fn setup(mut commands: Commands) {
    for i in 0..1000 {
        let agent = commands.spawn(ActionsBundle::default()).id();
        commands.actions(agent).add(CountdownAction::new(i));
    }
}

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
