use bevy::prelude::*;
use bevy_sequential_actions::*;

use actions::*;
use screenshots::*;

const SUPER_SILVER: Color = Color::srgb(0.933, 0.933, 0.933);
const EERIE_BLACK: Color = Color::srgb(0.141, 0.141, 0.137);
const GOLDENROD: Color = Color::srgb(0.823, 0.615, 0.129);

fn main() {
    App::new()
        .insert_resource(ClearColor(EERIE_BLACK))
        .insert_resource(TakeScreenshots(false))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: bevy::window::WindowResolution::new(512.0, 256.0),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ScreenshotsPlugin,
            ActionsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .run();
}

#[derive(Resource, Default)]
struct TakeScreenshots(bool);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0).mesh().resolution(1024)),
        material: materials.add(GOLDENROD),
        transform: Transform {
            translation: Vec3::NEG_Y * 1.0,
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::ONE,
        },
        ..default()
    });

    // Key light
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            color: Color::WHITE,
            intensity: 2_000_000.0,
            range: 20.0,
            radius: 0.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Back light
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            color: Color::WHITE,
            intensity: 1_000_000.0,
            range: 20.0,
            radius: 0.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 6.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Fill light
    commands.spawn(SpotLightBundle {
        spot_light: SpotLight {
            color: Color::WHITE,
            intensity: 1_000_000.0,
            range: 20.0,
            radius: 0.0,
            shadows_enabled: false,
            ..Default::default()
        },
        transform: Transform::from_xyz(-4.0, 2.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // Camera
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.5, 5.0)),
            ..Default::default()
        })
        .id();

    // Capsule guy
    let capsule_guy = commands
        .spawn(PbrBundle {
            mesh: meshes.add(Capsule3d::default()),
            material: materials.add(SUPER_SILVER),
            transform: Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ..Default::default()
        })
        .with_children(|builder| {
            let sphere = meshes.add(Sphere::default().mesh().uv(32, 18));
            let black = materials.add(Color::BLACK);

            let eye_left = Vec3::new(-0.2, 0.6, -0.4);
            let eye_right = Vec3::new(-eye_left.x, eye_left.y, eye_left.z);
            let eye_scale = Vec3::splat(0.25);

            builder.spawn(PbrBundle {
                mesh: sphere.clone(),
                material: black.clone(),
                transform: Transform {
                    translation: eye_left,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
            builder.spawn(PbrBundle {
                mesh: sphere,
                material: black,
                transform: Transform {
                    translation: eye_right,
                    scale: eye_scale,
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .id();

    // Executor
    let executor = commands.spawn(ActionsBundle::new()).id();
    commands.actions(executor).add_many(actions![
        WaitAction(1.0),
        RepeatActionSequence::new(
            u32::MAX,
            actions![
                |_agent, world: &mut World| {
                    if world.resource::<TakeScreenshots>().0 {
                        world.resource_mut::<TakeScreenshots>().0 = false;
                        world
                            .resource_mut::<NextState<ScreenshotState>>()
                            .set(ScreenshotState::Active);
                    }
                    true
                },
                WaitAction(0.5),
                LerpAction::new(
                    camera,
                    LerpValue::Transform(Transform {
                        translation: Vec3::new(0.0, 8.0, 9.0),
                        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                        scale: Vec3::ONE,
                    }),
                    1.0
                ),
                ParallelActions(actions![
                    LerpAction::new(capsule_guy, LerpValue::Position(Vec3::X * 3.0), 1.0),
                    LerpAction::new(
                        capsule_guy,
                        LerpValue::Rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                        0.5
                    ),
                ]),
                WaitAction(0.5),
                ParallelActions(actions![
                    LerpAction::new(capsule_guy, LerpValue::Position(Vec3::NEG_X * 3.0), 1.5),
                    LerpAction::new(
                        capsule_guy,
                        LerpValue::Rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                        0.5
                    ),
                ]),
                WaitAction(0.5),
                ParallelActions(actions![
                    LerpAction::new(capsule_guy, LerpValue::Position(Vec3::ZERO), 1.0),
                    LerpAction::new(
                        capsule_guy,
                        LerpValue::Rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                        0.5
                    ),
                ]),
                ParallelActions(actions![
                    LerpAction::new(camera, LerpValue::Position(Vec3::new(0.0, 0.5, 5.0)), 1.0),
                    LerpAction::new(camera, LerpValue::Rotation(Quat::IDENTITY), 1.0),
                    LerpAction::new(
                        capsule_guy,
                        LerpValue::Rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                        1.0
                    ),
                ]),
                WaitAction(0.5),
                |_agent, world: &mut World| {
                    world
                        .resource_mut::<NextState<ScreenshotState>>()
                        .set(ScreenshotState::None);
                    true
                },
            ]
        )
    ]);
}

fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut take_screenshots: ResMut<TakeScreenshots>,
    mut app_exit_event: ResMut<Events<AppExit>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        take_screenshots.0 = !take_screenshots.0;
        println!("Take screenshots: {}", take_screenshots.0);
    }

    if keys.just_pressed(KeyCode::Escape) {
        app_exit_event.send(AppExit::Success);
    }
}

mod screenshots {
    use bevy::{prelude::*, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};

    pub struct ScreenshotsPlugin;

    impl Plugin for ScreenshotsPlugin {
        fn build(&self, app: &mut App) {
            let fps = std::env::args()
                .skip(1)
                .next()
                .map(|s| s.parse::<u32>().ok())
                .flatten()
                .unwrap_or(24);
            println!("Screenshot framerate: {}", fps);

            std::fs::remove_dir_all("./screenshots").ok();
            std::fs::create_dir_all("./screenshots").unwrap();

            app.insert_resource(ScreenshotTimer(Timer::new(
                std::time::Duration::from_secs_f64(1.0 / fps as f64),
                TimerMode::Repeating,
            )))
            .insert_resource(ScreenshotFrame(0))
            .init_state::<ScreenshotState>()
            .add_systems(OnEnter(ScreenshotState::Active), on_enter_screenshot_state)
            .add_systems(
                Update,
                take_screenshots.run_if(in_state(ScreenshotState::Active)),
            );
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
    pub enum ScreenshotState {
        #[default]
        None,
        Active,
    }

    #[derive(Resource)]
    pub struct ScreenshotTimer(Timer);

    #[derive(Resource)]
    pub struct ScreenshotFrame(u32);

    fn on_enter_screenshot_state(
        mut timer: ResMut<ScreenshotTimer>,
        mut frame: ResMut<ScreenshotFrame>,
    ) {
        timer.0.reset();
        frame.0 = 0;
    }

    fn take_screenshots(
        mut timer: ResMut<ScreenshotTimer>,
        mut frame: ResMut<ScreenshotFrame>,
        mut screenshot_manager: ResMut<ScreenshotManager>,
        primary_window: Query<Entity, With<PrimaryWindow>>,
        time: Res<Time>,
    ) {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            screenshot_manager
                .save_screenshot_to_disk(
                    primary_window.single(),
                    format!("screenshots/frame{:04}.png", frame.0),
                )
                .unwrap();
            frame.0 += 1;
        }
    }
}

mod actions {
    use bevy::prelude::*;
    use bevy_sequential_actions::*;

    pub struct ActionsPlugin;

    impl Plugin for ActionsPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugins(SequentialActionsPlugin)
                .add_systems(Update, (wait, lerp));
        }
    }

    pub struct RepeatActionSequence<const N: usize> {
        actions: [BoxedAction; N],
        index: usize,
        repeat: u32,
    }

    impl<const N: usize> RepeatActionSequence<N> {
        pub const fn new(repeat: u32, actions: [BoxedAction; N]) -> Self {
            Self {
                actions,
                index: 0,
                repeat,
            }
        }
    }

    impl<const N: usize> Action for RepeatActionSequence<N> {
        fn is_finished(&self, agent: Entity, world: &World) -> bool {
            self.actions[self.index].is_finished(agent, world)
        }

        fn on_add(&mut self, agent: Entity, world: &mut World) {
            self.actions
                .iter_mut()
                .for_each(|action| action.on_add(agent, world));
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            self.actions[self.index].on_start(agent, world)
        }

        fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
            self.actions[self.index].on_stop(agent, world, reason);

            if reason == StopReason::Canceled {
                self.index = self.actions.len();
            }
        }

        fn on_drop(mut self: Box<Self>, agent: Entity, world: &mut World, reason: DropReason) {
            match reason {
                DropReason::Done => {
                    self.index += 1;

                    if self.index >= self.actions.len() {
                        if self.repeat == 0 {
                            self.actions
                                .iter_mut()
                                .for_each(|action| action.on_remove(agent, world));
                            return;
                        }

                        self.repeat -= 1;
                        self.index = 0;
                        world.get_mut::<ActionQueue>(agent).unwrap().push_back(self);
                        return;
                    }

                    world
                        .get_mut::<ActionQueue>(agent)
                        .unwrap()
                        .push_front(self);
                }
                DropReason::Skipped | DropReason::Cleared => {
                    self.actions
                        .iter_mut()
                        .for_each(|action| action.on_remove(agent, world));
                }
            }
        }
    }

    pub struct ParallelActions<const N: usize>(pub [BoxedAction; N]);

    impl<const N: usize> Action for ParallelActions<N> {
        fn is_finished(&self, agent: Entity, world: &World) -> bool {
            self.0.iter().all(|action| action.is_finished(agent, world))
        }

        fn on_add(&mut self, agent: Entity, world: &mut World) {
            self.0
                .iter_mut()
                .for_each(|action| action.on_add(agent, world));
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            std::array::from_fn::<bool, N, _>(|i| self.0[i].on_start(agent, world))
                .into_iter()
                .all(|b| b)
        }

        fn on_stop(&mut self, agent: Entity, world: &mut World, reason: StopReason) {
            self.0
                .iter_mut()
                .for_each(|action| action.on_stop(agent, world, reason));
        }

        fn on_remove(&mut self, agent: Entity, world: &mut World) {
            self.0
                .iter_mut()
                .for_each(|action| action.on_remove(agent, world));
        }
    }

    pub struct WaitAction(pub f32);

    impl Action for WaitAction {
        fn is_finished(&self, agent: Entity, world: &World) -> bool {
            world.get::<WaitTimer>(agent).unwrap().0 <= 0.0
        }

        fn on_start(&mut self, agent: Entity, world: &mut World) -> bool {
            world.entity_mut(agent).insert(WaitTimer(self.0));
            false
        }

        fn on_stop(&mut self, agent: Entity, world: &mut World, _reason: StopReason) {
            world.entity_mut(agent).remove::<WaitTimer>();
        }
    }

    #[derive(Component)]
    struct WaitTimer(f32);

    fn wait(mut wait_q: Query<&mut WaitTimer>, time: Res<Time>) {
        for mut wait_timer in &mut wait_q {
            wait_timer.0 -= time.delta_seconds();
        }
    }

    pub struct LerpAction {
        pub target: Entity,
        pub value: LerpValue,
        pub duration: f32,
        executor: Entity,
    }

    impl LerpAction {
        pub const fn new(target: Entity, value: LerpValue, duration: f32) -> Self {
            Self {
                target,
                value,
                duration,
                executor: Entity::PLACEHOLDER,
            }
        }
    }

    pub enum LerpValue {
        Position(Vec3),
        Rotation(Quat),
        Transform(Transform),
    }

    #[derive(Component)]
    struct LerpTarget(Entity);

    #[derive(Component)]
    enum LerpValues {
        Position(Vec3, Vec3),
        Rotation(Quat, Quat),
        Transform(Transform, Transform),
    }

    #[derive(Component)]
    struct LerpTimer(Timer);

    #[derive(Bundle)]
    struct LerpBundle {
        target: LerpTarget,
        values: LerpValues,
        timer: LerpTimer,
    }

    impl Action for LerpAction {
        fn is_finished(&self, _agent: Entity, world: &World) -> bool {
            world.get::<LerpTimer>(self.executor).unwrap().0.finished()
        }

        fn on_start(&mut self, _agent: Entity, world: &mut World) -> bool {
            let transform = world.get::<Transform>(self.target).unwrap();
            let values = match self.value {
                LerpValue::Position(v) => LerpValues::Position(transform.translation, v),
                LerpValue::Rotation(q) => LerpValues::Rotation(transform.rotation, q),
                LerpValue::Transform(t) => LerpValues::Transform(transform.clone(), t),
            };
            self.executor = world
                .spawn(LerpBundle {
                    target: LerpTarget(self.target),
                    values,
                    timer: LerpTimer(Timer::from_seconds(self.duration, TimerMode::Once)),
                })
                .id();
            false
        }

        fn on_stop(&mut self, _agent: Entity, world: &mut World, _reason: StopReason) {
            world.despawn(self.executor);
        }
    }

    fn lerp(
        mut lerp_q: Query<(&mut LerpTimer, &LerpTarget, &LerpValues)>,
        mut transform_q: Query<&mut Transform>,
        time: Res<Time>,
    ) {
        for (mut timer, target, values) in &mut lerp_q {
            timer.0.tick(time.delta());

            let t = timer.0.fraction();
            let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

            let mut transform = transform_q.get_mut(target.0).unwrap();

            match values {
                LerpValues::Position(v1, v2) => {
                    transform.translation = Vec3::lerp(*v1, *v2, smoothstep);
                }
                LerpValues::Rotation(q1, q2) => {
                    transform.rotation = Quat::slerp(*q1, *q2, smoothstep);
                }
                LerpValues::Transform(t1, t2) => {
                    transform.translation = Vec3::lerp(t1.translation, t2.translation, smoothstep);
                    transform.rotation = Quat::slerp(t1.rotation, t2.rotation, smoothstep);
                    transform.scale = Vec3::lerp(t1.scale, t2.scale, smoothstep);
                }
            }
        }
    }
}
