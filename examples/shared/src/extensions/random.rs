use bevy::prelude::*;

pub trait RandomExt {
    type Bound;

    fn random(min: Self::Bound, max: Self::Bound) -> Self;
}

impl RandomExt for f32 {
    type Bound = Self;

    fn random(min: Self::Bound, max: Self::Bound) -> Self {
        assert!(min <= max);
        assert!(min + 0.0 * (max - min) == min);
        assert!(min + 1.0 * (max - min) == max);

        min + fastrand::f32() * (max - min)
    }
}

impl RandomExt for Vec3 {
    type Bound = Self;

    fn random(min: Self::Bound, max: Self::Bound) -> Self {
        let x = f32::random(min.x, max.x);
        let y = f32::random(min.y, max.y);
        let z = f32::random(min.z, max.z);
        Self::new(x, y, z)
    }
}

impl RandomExt for Quat {
    type Bound = Vec3;

    fn random(euler_min: Self::Bound, euler_max: Self::Bound) -> Self {
        let r = Vec3::random(euler_min, euler_max);
        Self::from_euler(EulerRot::XYZ, r.x, r.y, r.z)
    }
}
