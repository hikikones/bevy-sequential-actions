use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ExecutorKind};
use bevy_sequential_actions::*;

use criterion::{criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, many_countdowns);

fn many_countdowns(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_countdowns");
    group.sample_size(10);

    for agents in [100, 10_000, 1_000_000] {
        group.bench_function(format!("{agents}"), |b| {
            b.iter(|| run_many_countdowns(agents));
        });
    }

    group.finish();
}

fn run_many_countdowns(agents: u32) {
    let mut app = App::empty();
    app.edit_schedule(Main, |schedule| {
        schedule
            .set_executor_kind(ExecutorKind::SingleThreaded)
            .add_systems((
                countdown,
                SequentialActionsPlugin::check_actions::<()>.after(countdown),
            ));
    });

    for i in 0..agents {
        let agent = app.world.spawn(ActionsBundle::new()).id();
        app.world.actions(agent).add(CountdownAction {
            count: i,
            current: None,
        });
    }

    for _ in 0..10 {
        app.update();
    }

    struct CountdownAction {
        count: u32,
        current: Option<u32>,
    }

    impl Action for CountdownAction {
        fn is_finished(&self, agent: Entity, world: &World) -> bool {
            world.get::<Countdown>(agent).unwrap().0 == 0
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            let count = self.current.take().unwrap_or(self.count);
            world.entity_mut(agent).insert(Countdown(count));
            self.is_finished(agent, world)
        }

        fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
            let countdown = world.entity_mut(agent).take::<Countdown>();
            if reason == StopReason::Paused {
                self.current = countdown.unwrap().0.into();
            }
        }
    }

    #[derive(Component)]
    struct Countdown(u32);

    fn countdown(mut countdown_q: Query<&mut Countdown>) {
        for mut countdown in &mut countdown_q {
            countdown.0 -= 1;
        }
    }
}
