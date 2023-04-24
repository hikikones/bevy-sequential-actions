use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use bevy_sequential_actions::*;

criterion_group!(benches, many_countdowns);
criterion_main!(benches);

fn many_countdowns(c: &mut Criterion) {
    c.bench_function("many_countdowns", |bencher| {
        bencher.iter(|| black_box(run_many_countdowns()));
    });
}

fn run_many_countdowns() {
    const MAX: i32 = 100;

    let mut app = App::new();
    app.add_plugin(SequentialActionsPlugin::default())
        .add_startup_system(|mut commands: Commands| {
            for i in 0..MAX {
                let agent = commands.spawn(ActionsBundle::default()).id();
                commands.actions(agent).add(CountdownAction::new(i));
            }
        })
        .add_system(countdown);

    for _ in 0..MAX {
        app.update();
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
