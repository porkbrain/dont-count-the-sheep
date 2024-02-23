#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod gi;
pub mod prelude;

use std::marker::PhantomData;

use bevy::ecs::component::Component;
pub use gi::Plugin;

#[derive(Component, Default)]
pub struct SceneCamera<T>(PhantomData<T>);
