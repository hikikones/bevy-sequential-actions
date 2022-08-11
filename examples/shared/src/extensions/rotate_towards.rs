use bevy::prelude::*;

pub trait RotateTowardsExt {
    type Target;

    fn rotate_towards(&mut self, target: Self::Target, max_radians_delta: f32) -> bool;
}

impl RotateTowardsExt for Quat {
    type Target = Self;

    fn rotate_towards(&mut self, target: Self, max_radians_delta: f32) -> bool {
        assert!(self.is_normalized());
        assert!(target.is_normalized());

        let angle = self.angle_between(target);

        if angle == 0.0 {
            *self = target;
            return true;
        }

        *self = self.slerp(target, (max_radians_delta / angle).min(1.0));

        false
    }
}

impl RotateTowardsExt for Transform {
    type Target = Quat;

    fn rotate_towards(&mut self, target: Self::Target, max_radians_delta: f32) -> bool {
        let mut rot = self.rotation;
        let reached_target = rot.rotate_towards(target, max_radians_delta);
        self.rotation = rot;

        reached_target
    }
}
