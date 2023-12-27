#![allow(clippy::type_complexity)]

pub mod gi;
pub mod prelude;

pub use gi::Plugin;

use bevy::ecs::component::Component;
use std::marker::PhantomData;

#[derive(Component, Default)]
pub struct SceneCamera<T>(PhantomData<T>);
