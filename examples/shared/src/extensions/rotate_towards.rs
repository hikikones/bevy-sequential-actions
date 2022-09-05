use bevy::prelude::*;

pub trait RotateTowardsExt {
    fn rotate_towards(self, target: Self, max_radians_delta: f32) -> Self;
}

impl RotateTowardsExt for Quat {
    fn rotate_towards(self, target: Self, max_radians_delta: f32) -> Self {
        assert!(self.is_normalized());
        assert!(target.is_normalized());

        let angle = self.angle_between(target);

        if angle == 0.0 {
            return target;
        }

        self.slerp(target, (max_radians_delta / angle).min(1.0))
    }
}

pub trait RotateTowardsTransformExt {
    fn rotate_towards(&mut self, target: Quat, max_radians_delta: f32);
}

impl RotateTowardsTransformExt for Transform {
    fn rotate_towards(&mut self, target: Quat, max_radians_delta: f32) {
        self.rotation = self.rotation.rotate_towards(target, max_radians_delta);
    }
}
