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
