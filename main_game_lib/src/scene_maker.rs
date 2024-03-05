//! Loads scenes from asset config file.
//! Useful for creating scenes in-game with dev tools and then loading them
//! later in the game.
//! This suggests that some of the code is not used in the final build and
//! hidden behind a feature `devtools`.
//!
//! # Example
//! You can spawn a scene with a system like this:
//!
//! ```rust,ignore
//! fn load_scene<T: SpriteScene>(
//!     mut cmd: Commands,
//!     asset_server: Res<AssetServer>,
//!     mut scenes: ResMut<Assets<SceneSerde>>,
//!     mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
//! ) {
//!     let mut scene = scenes
//!         .remove(asset_server.load(T::asset_path()))
//!         .expect("is_scene_file_ready run condition");
//!
//!     while let Some((entity_cmd, name)) =
//!         scene.spawn_next_sprite(&mut cmd, &asset_server, &mut atlas_layouts)
//!     {
//!         // your custom logic for the sprite
//!     }
//! }
//! ```
//!
//! Make sure to use the [`bevy::ecs::schedule::common_conditions::not`] ∘
//! [`are_scene_sprites_spawned`].

pub(crate) mod store_and_load;
#[cfg(feature = "devtools")]
mod toolbar;

use bevy::ecs::{entity::Entity, query::With, system::Query};
#[cfg(feature = "devtools")]
pub use plugin::Plugin;
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

#[cfg(feature = "devtools")]
mod plugin {
    use bevy::{
        app::{App, Update},
        ecs::schedule::{
            common_conditions::in_state, IntoSystemConfigs, OnEnter, OnExit,
            States,
        },
    };
    use common_action::interaction_just_pressed;

    use super::*;
    use crate::StateSemantics;

    /// A plugin that allows in-game scene creation and editing.
    pub struct Plugin<T, S> {
        states: StateSemantics<S>,
        _phantom: std::marker::PhantomData<T>,
    }

    impl<T, S> Plugin<T, S> {
        /// Create a new scene maker plugin.
        pub fn new(states: StateSemantics<S>) -> Self {
            Self {
                states,
                _phantom: std::marker::PhantomData,
            }
        }
    }

    impl<T: SpriteScene, S: States + Copy> bevy::app::Plugin for Plugin<T, S> {
        fn build(&self, app: &mut App) {
            use bevy_inspector_egui::quick::ResourceInspectorPlugin;

            use self::toolbar::SceneMakerToolbar;

            app.register_type::<SceneMakerToolbar>().add_plugins(
                ResourceInspectorPlugin::<SceneMakerToolbar>::default(),
            );

            app.add_systems(
                Update,
                store_and_load::store
                    .run_if(in_state(self.states.running))
                    .run_if(interaction_just_pressed()),
            );

            app.add_systems(OnEnter(self.states.loading), toolbar::spawn)
                .add_systems(
                    Update,
                    (
                        toolbar::move_sprite_system,
                        store_and_load::react_to_changes::<T>,
                    )
                        .run_if(in_state(self.states.running)),
                )
                .add_systems(OnExit(self.states.quitting), toolbar::despawn);
        }
    }
}
