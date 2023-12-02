pub(crate) use bevy::prelude::*;

#[derive(Default, Deref, DerefMut, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Radians(f32);

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Velocity(Vec2);

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct AngularVelocity(pub f32);

impl Radians {
    pub(crate) fn new(radians: f32) -> Self {
        Self(radians)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub(crate) enum Direction {
    #[allow(dead_code)]
    None,
    Left,
    Right,
}

impl Direction {
    pub(crate) fn sign(&self) -> f32 {
        match self {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            Direction::None => 0.0,
        }
    }

    pub(crate) fn is_aligned(&self, point: f32) -> bool {
        match self {
            Self::Left => point < 0.0,
            Self::Right => point > 0.0,
            Self::None => point == 0.0,
        }
    }
}
