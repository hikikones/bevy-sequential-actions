use bevy::prelude::*;

pub mod command_action;
pub mod despawn_action;
pub mod move_action;
pub mod quit_action;
pub mod rotate_action;
pub mod wait_action;

pub use command_action::*;
pub use despawn_action::*;
pub use move_action::*;
pub use quit_action::*;
pub use rotate_action::*;
pub use wait_action::*;

/// Stage for running actions.
///
/// Useful for avoiding ambiguous system ordering when modifying actions.
///
/// # Example
///
/// Say you have an entity with a running action `A` and another action `B` in the queue.
/// Say you also have a system that stops the current action when `space` is pressed.
/// Say also that everything runs in the same stage, and no explicit system ordering has been done.
/// You want to stop this action before it finishes, so you press `space`.
///
/// And so the question appears, what will happen?
///
/// We don't know, but here are two possibilities:
///
/// * Action `A` is stopped before it finishes.
/// * Action `A` is finished, and _then_ the stop command is applied, effectively stopping the next action `B`.
///
/// The latter is usually not what we want. Running all actions in a custom stage alleviates this problem.
const ACTIONS_STAGE: &str = "update_actions";

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(CoreStage::Update, ACTIONS_STAGE, SystemStage::parallel())
            .add_plugin(WaitActionPlugin)
            .add_plugin(MoveActionPlugin)
            .add_plugin(RotateActionPlugin);
    }
}

fn random_f32(min: f32, max: f32) -> f32 {
    assert!(min <= max);
    assert!(min + 0.0 * (max - min) == min);
    assert!(min + 1.0 * (max - min) == max);

    min + fastrand::f32() * (max - min)
}

fn random_vec3(min: Vec3, max: Vec3) -> Vec3 {
    let x = random_f32(min.x, max.x);
    let y = random_f32(min.y, max.y);
    let z = random_f32(min.z, max.z);
    Vec3::new(x, y, z)
}

fn random_quat(euler_min: Vec3, euler_max: Vec3) -> Quat {
    let x = random_f32(euler_min.x, euler_max.x);
    let y = random_f32(euler_min.y, euler_max.y);
    let z = random_f32(euler_min.z, euler_max.z);
    Quat::from_euler(EulerRot::XYZ, x, y, z)
}
