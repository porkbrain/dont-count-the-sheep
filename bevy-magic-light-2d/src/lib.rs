use std::marker::PhantomData;

use bevy::prelude::*;

pub mod gi;
pub mod prelude;

#[derive(Component, Default)]
pub struct SceneCamera<T>(PhantomData<T>);
