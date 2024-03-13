//! Loads scenes from asset config file.
//! Useful for creating scenes in-game with dev tools and then loading them
//! later in the game.
//! This suggests that some of the code is not used in the final build and
//! hidden behind a feature `devtools`.
//!
//! See the existing spawn implementations for examples.
//! Make sure to use the [`bevy::ecs::schedule::common_conditions::not`] ∘
//! [`are_sprites_spawned_and_file_despawned`] run condition on your spawn
//! system.

pub(crate) mod store_and_load;

use bevy::ecs::{entity::Entity, query::With, system::Query};
pub use store_and_load::{
    LoadedFromSceneFile, SceneSerde, SceneSpriteAtlas, SceneSpriteConfig,
    SpriteScene, SpriteSceneHandle,
};

/// Use this as a `run_if` cond to prevent loading the scene multiple times.
pub fn are_sprites_spawned_and_file_despawned<T: SpriteScene>() -> impl FnMut(
    Query<Entity, With<LoadedFromSceneFile>>,
    Query<Entity, With<SpriteSceneHandle<T>>>,
)
    -> bool {
    move |entities, handle| !entities.is_empty() && handle.is_empty()
}
