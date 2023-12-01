pub(crate) use bevy::prelude::*;

use bevy::sprite::MaterialMesh2dBundle;

#[derive(Component, Default, Deref, DerefMut)]
pub(crate) struct Velocity(Vec2);

#[derive(Bundle, Default)]
pub(crate) struct BodyBundle {
    pub(crate) mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub(crate) velocity: Velocity,
}
