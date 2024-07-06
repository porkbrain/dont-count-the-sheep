//! Bevy attempts to load any file in the asset directory when loading folders.
//! However, there are file patterns we want to ignore.
//! Add extensions to this loader to skip file loading.
//!
//! <https://github.com/bevyengine/bevy/pull/11214#issuecomment-1996004344>

use std::convert::Infallible;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    utils::ConditionalSendFuture,
};

/// Files loaded by this loader are ignored.
/// The bytes are not polled from the reader.
#[derive(Debug, Default)]
pub struct Loader;

impl AssetLoader for Loader {
    type Asset = ();
    type Settings = ();
    type Error = Infallible;

    fn load<'a>(
        &'a self,
        _reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<
        Output = Result<
            <Self as AssetLoader>::Asset,
            <Self as AssetLoader>::Error,
        >,
    > {
        Box::pin(async move { Ok(()) })
    }

    fn extensions(&self) -> &[&str] {
        &["import", "log"]
    }
}
