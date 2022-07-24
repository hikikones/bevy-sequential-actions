use bevy::prelude::*;

pub trait MoveTowardsExt {
    type Target;

    fn move_towards(&mut self, target: Self::Target, max_delta: f32) -> bool;
}

impl MoveTowardsExt for f32 {
    type Target = Self;

    fn move_towards(&mut self, target: Self, max_delta: f32) -> bool {
        assert!(max_delta > 0.0);

        if (target - *self).abs() <= max_delta {
            *self = target;
            return true;
        }

        *self += (target - *self).signum() * max_delta;

        false
    }
}

impl MoveTowardsExt for Vec3 {
    type Target = Self;

    fn move_towards(&mut self, target: Self, max_delta: f32) -> bool {
        assert!(max_delta > 0.0);

        if self.distance(target) <= max_delta {
            *self = target;
            return true;
        }

        *self += (target - *self).normalize() * max_delta;

        false
    }
}

impl MoveTowardsExt for Transform {
    type Target = Vec3;

    fn move_towards(&mut self, target: Self::Target, max_delta: f32) -> bool {
        let mut pos = self.translation;
        let reached_target = pos.move_towards(target, max_delta);
        self.translation = pos;

        reached_target
    }
}
