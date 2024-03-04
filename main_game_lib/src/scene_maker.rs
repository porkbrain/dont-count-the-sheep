//! Loads scenes from asset config file.
//! Useful for creating scenes in-game with dev tools and then loading them
//! later in the game.
//! This suggests that some of the code is not used in the final build and
//! hidden behind a feature `devtools`.

mod store_and_load;
#[cfg(feature = "devtools")]
mod toolbar;

use self::{
    store_and_load::{SceneSprite, SceneSpriteAtlas},
    toolbar::SceneMakerToolbar,
};
use crate::prelude::*;

/// A plugin that allows in-game scene creation and editing.
pub struct Plugin<S> {
    /// The state when the scene is loading.
    /// Setups up resources.
    pub loading: S,
    /// The state when the scene is running.
    /// Runs the necessary systems.
    pub running: S,
    /// The state when the scene is quitting.
    /// Cleans up resources.
    pub quitting: S,
}

impl<S: States + Copy> bevy::app::Plugin for Plugin<S> {
    fn build(&self, app: &mut App) {
        app.register_type::<SceneSprite>()
            .register_type::<SceneSpriteAtlas>()
            .add_systems(
                Update,
                store_and_load::load_scene.run_if(in_state(self.loading)),
            );

        #[cfg(feature = "devtools")]
        {
            use bevy_inspector_egui::quick::ResourceInspectorPlugin;

            app.register_type::<SceneMakerToolbar>().add_plugins(
                ResourceInspectorPlugin::<SceneMakerToolbar>::default(),
            );

            app.add_systems(OnEnter(self.loading), toolbar::spawn)
                .add_systems(
                    Update,
                    (
                        toolbar::move_sprite_system,
                        store_and_load::react_to_changes,
                    )
                        .run_if(in_state(self.running)),
                )
                .add_systems(OnExit(self.quitting), toolbar::despawn);
        }
    }
}
