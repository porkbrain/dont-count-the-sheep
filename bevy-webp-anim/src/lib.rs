#![doc = include_str!("../README.md")]

mod loader;
mod systems;
mod types;

pub use loader::*;
pub use types::*;

use bevy::{
    app::{App, FixedUpdate},
    asset::AssetApp,
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<WebpLoader>()
            .init_asset::<WebpAnimation>()
            .add_systems(FixedUpdate, systems::load_next_frame);
    }

    fn finish(&self, _app: &mut App) {
        //
    }
}
