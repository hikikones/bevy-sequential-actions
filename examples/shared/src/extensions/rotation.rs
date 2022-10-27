use bevy::prelude::*;

pub trait FromLookExt {
    fn from_look(direction: Vec3, up: Vec3) -> Self;
}

impl FromLookExt for Quat {
    fn from_look(direction: Vec3, up: Vec3) -> Self {
        assert!(direction != Vec3::ZERO);
        assert!(up != Vec3::ZERO);

        let forward = Vec3::normalize(-direction);
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        Self::from_mat3(&Mat3::from_cols(right, up, forward))
    }
}

pub trait FromEulerXYZExt {
    fn from_euler_xyz(v: Vec3) -> Self;
}

impl FromEulerXYZExt for Quat {
    fn from_euler_xyz(v: Vec3) -> Self {
        Self::from_euler(EulerRot::XYZ, v.x, v.y, v.z)
    }
}

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
