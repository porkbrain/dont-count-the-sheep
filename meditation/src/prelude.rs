pub(crate) use crate::consts;
pub(crate) use bevy::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Acceleration(pub Vec2);
#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Velocity(pub Vec2);

#[derive(Bundle, Default)]
pub(crate) struct BodyBundle {
    pub(crate) mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) acceleration: Acceleration,
    pub(crate) velocity: Velocity,
}
