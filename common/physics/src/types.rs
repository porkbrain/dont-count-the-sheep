use bevy::prelude::*;

#[derive(Default, Deref, DerefMut, Debug, Clone, Copy, PartialEq)]
pub struct Radians(f32);

#[derive(
    Component, Default, Deref, DerefMut, Clone, Copy, PartialEq, Debug,
)]
pub struct Velocity(Vec2);

/// Positive should be counter-clockwise.
#[derive(
    Component, Default, Deref, DerefMut, Clone, Copy, PartialEq, Debug,
)]
pub struct AngularVelocity(pub f32);

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum MotionDirection {
    #[allow(dead_code)]
    None,
    Left,
    Right,
}

impl MotionDirection {
    #[inline]
    pub fn sign(&self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
            Self::None => 0.0,
        }
    }

    #[inline]
    pub fn is_aligned(&self, point: f32) -> bool {
        match self {
            Self::Left => point < 0.0,
            Self::Right => point > 0.0,
            Self::None => point == 0.0,
        }
    }
}

impl Radians {
    #[inline]
    pub fn new(radians: f32) -> Self {
        Self(radians)
    }
}

impl Velocity {
    #[inline]
    pub fn new(v: Vec2) -> Self {
        Self(v)
    }
}

impl From<Vec2> for Velocity {
    #[inline]
    fn from(vec: Vec2) -> Self {
        Self(vec)
    }
}

impl From<f32> for AngularVelocity {
    #[inline]
    fn from(radians: f32) -> Self {
        Self(radians)
    }
}

impl AngularVelocity {
    #[inline]
    pub fn new(radians: f32) -> Self {
        Self(radians)
    }
}
