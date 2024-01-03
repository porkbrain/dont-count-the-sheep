#![allow(clippy::type_complexity)]

pub mod gi;
pub mod prelude;

use std::marker::PhantomData;

use bevy::ecs::component::Component;
pub use gi::Plugin;

#[derive(Component, Default)]
pub struct SceneCamera<T>(PhantomData<T>);
