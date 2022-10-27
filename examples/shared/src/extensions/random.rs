use bevy::prelude::*;

pub trait RandomExt {
    fn random(min: Self, max: Self) -> Self;
}

impl RandomExt for f32 {
    fn random(min: Self, max: Self) -> Self {
        assert!(min <= max);
        assert!(min + 0.0 * (max - min) == min);
        assert!(min + 1.0 * (max - min) == max);

        min + fastrand::f32() * (max - min)
    }
}

impl RandomExt for Vec3 {
    fn random(min: Self, max: Self) -> Self {
        let x = f32::random(min.x, max.x);
        let y = f32::random(min.y, max.y);
        let z = f32::random(min.z, max.z);
        Self::new(x, y, z)
    }
}
