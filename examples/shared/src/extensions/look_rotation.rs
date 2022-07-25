use bevy::prelude::*;

pub trait LookRotationExt {
    fn look_rotation(direction: Vec3, up: Vec3) -> Self;
}

impl LookRotationExt for Quat {
    fn look_rotation(direction: Vec3, up: Vec3) -> Self {
        assert!(direction != Vec3::ZERO);
        assert!(up != Vec3::ZERO);

        let forward = Vec3::normalize(-direction);
        let right = up.cross(forward).normalize();
        let up = forward.cross(right);
        Self::from_mat3(&Mat3::from_cols(right, up, forward))
    }
}
