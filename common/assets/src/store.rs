use std::{marker::PhantomData, sync::Arc};

use bevy::{
    asset::{AssetServer, Handle, StrongHandle, UntypedHandle},
    ecs::system::{Commands, Res, Resource},
    log::error,
    prelude::default,
    utils::HashMap,
};

/// Assets that are loaded once and never unloaded.
pub struct StaticScene;

/// Assets that are loaded once and never unloaded.
pub type StaticAssetStore = AssetStore<StaticScene>;

impl AssetList for StaticScene {
    fn folders() -> &'static [&'static str] {
        &[super::paths::fonts::FOLDER]
    }
}

/// When all [`StrongHandle`]s are dropped, an asset is unloaded.
///
/// However, we often find ourselves reusing the same asset in multiple places.
/// This can lead to a scenario where we load and drop the same asset over and
/// over again.
///
/// Hence, we organize scene assets around the [`AssetStore`] where `T` is the
/// generic for a given scene.
/// Then, we can load all assets on scene load and drop them all on scene exit.
#[derive(Resource)]
pub struct AssetStore<T> {
    assets: HashMap<&'static str, Arc<StrongHandle>>,

    _phantom: PhantomData<T>,
}

/// Implement this for your scene marking trait and use the
/// [`insert_as_resource`] and [`remove_as_resource`] systems.
pub trait AssetList {
    fn folders() -> &'static [&'static str] {
        &[]
    }

    fn files() -> &'static [&'static str] {
        &[]
    }
}

/// Inserts a resource that holds strong handles to dialog assets.
/// In another words, begins loading process for dialog assets and
/// it keeps the assets loaded.
pub fn insert_as_resource<T: AssetList + Send + Sync + 'static>(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
) {
    cmd.insert_resource(AssetStore::<T>::load_all(&asset_server));
}

/// Removes the asset store that holds strong handles to given assets.
/// Then they will be unloaded.
pub fn remove_as_resource<T: Send + Sync + 'static>(mut cmd: Commands) {
    cmd.remove_resource::<AssetStore<T>>();
}

impl<T: AssetList> AssetStore<T> {
    pub fn load_all(asset_server: &bevy::asset::AssetServer) -> Self {
        let mut store = Self::new();

        for folder in T::folders() {
            match asset_server.load_folder(*folder) {
                Handle::Strong(h) => {
                    store.assets.insert(folder, h);
                }
                Handle::Weak(_) => error!("Cannot append weak handle"),
            }
        }

        for file in T::files() {
            if let Handle::Strong(handle) = asset_server.load_untyped(*file) {
                store.assets.insert(file, handle);
            }
        }

        store
    }

    pub fn are_all_loaded(
        &self,
        asset_server: &bevy::asset::AssetServer,
    ) -> bool {
        self.assets.values().all(|h| {
            asset_server.is_loaded_with_dependencies(&UntypedHandle::Strong(
                Arc::clone(h),
            ))
        })
    }
}

impl<T> AssetStore<T> {
    pub fn new() -> Self {
        Self {
            assets: default(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for AssetStore<T> {
    fn default() -> Self {
        Self::new()
    }
}
