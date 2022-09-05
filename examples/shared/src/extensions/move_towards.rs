use bevy::prelude::*;

pub trait MoveTowardsExt {
    fn move_towards(self, target: Self, max_delta: f32) -> Self;
}

impl MoveTowardsExt for f32 {
    fn move_towards(self, target: Self, max_delta: f32) -> Self {
        assert!(max_delta > 0.0);

        if (target - self).abs() <= max_delta {
            return target;
        }

        self + (target - self).signum() * max_delta
    }
}

impl MoveTowardsExt for Vec3 {
    fn move_towards(self, target: Self, max_delta: f32) -> Self {
        assert!(max_delta > 0.0);

        if self.distance(target) <= max_delta {
            return target;
        }

        self + (target - self).normalize() * max_delta
    }
}

pub trait MoveTowardsTransformExt {
    fn move_towards(&mut self, target: Vec3, max_delta: f32);
}

impl MoveTowardsTransformExt for Transform {
    fn move_towards(&mut self, target: Vec3, max_delta: f32) {
        self.translation = self.translation.move_towards(target, max_delta);
    }
}
