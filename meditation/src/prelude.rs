pub(crate) use crate::consts;
pub(crate) use bevy::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Acceleration(Vec2);
#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Velocity(Vec2);

#[derive(Bundle, Default)]
pub(crate) struct BodyBundle {
    pub(crate) mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) acceleration: Acceleration,
    pub(crate) velocity: Velocity,
}

impl Acceleration {
    pub(crate) fn new(v: Vec2) -> Self {
        Self(v)
    }
}
