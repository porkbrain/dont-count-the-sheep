//! Dialog is a cyclic directed graph of two kinds of nodes:
//! - Vocative nodes, which are the actual dialog lines.
//! - Guard nodes, which are nodes that mutate game state and serve as
//!   middleware in the dialog

mod deser;
mod list;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    reflect::Reflect,
    utils::hashbrown::HashMap,
};
pub use list::DialogRoot;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Character;

/// The dialog asset that can be started.
/// Since dialogs can be stateful, state is lazy loaded.
#[derive(Asset, Deserialize, Serialize, Reflect)]
pub struct Dialog {
    pub(crate) nodes: HashMap<NodeName, Node>,
    pub(crate) root: NodeName,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub(crate) struct Node {
    pub(crate) name: NodeName,
    pub(crate) who: Character,
    pub(crate) kind: NodeKind,
    pub(crate) next: Vec<NodeName>,
}

#[derive(
    Debug, Deserialize, Serialize, Reflect, Clone, Hash, PartialEq, Eq,
)]
pub(crate) enum NodeName {
    Explicit(String),
    Auto(usize),
    EndDialog,
    Emerge,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub(crate) enum NodeKind {
    Guard {
        /// Guard states are persisted across dialog sessions if
        /// - the node has a [`NodeName::Explicit`]
        ///
        /// Otherwise the state is discarded after the dialog is over.
        state: Guard,
        #[reflect(ignore)]
        params: HashMap<String, toml::Value>,
    },
    Vocative {
        /// The dialog line to print.
        line: String,
    },
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub(crate) enum Guard {
    EndDialog,
    Emerge,
    ExhaustiveAlternatives(LazyGuardState<ExhaustiveAlternativesState>),
}

/// Loading state for each guard is unnecessary until we actually prompt the
/// guard.
/// This abstraction allows us to defer loading.
#[derive(Debug, Default, Deserialize, Serialize, Reflect)]
pub(crate) enum LazyGuardState<T> {
    Ready(T),
    #[default]
    Load,
}

#[derive(Debug, Serialize, Deserialize, Default, Reflect)]
pub(crate) struct ExhaustiveAlternativesState {
    pub(crate) next_to_show: usize,
}

/// Loads .toml files into [`Dialog`] representation.
#[derive(Default)]
pub(crate) struct DialogLoader;

/// Errors that can occur when loading assets from .toml files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub(crate) enum LoaderError {
    /// The file could not be loaded, most likely not found.
    #[error("Could load .toml file: {0}")]
    Io(#[from] std::io::Error),
    /// We convert the file bytes into a string, which can fail.
    #[error("Non-utf8 string in .toml file: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    /// The .toml file could not be parsed.
    #[error("Error parsing .toml file: {0}")]
    Toml(#[from] toml::de::Error),
}

impl AssetLoader for DialogLoader {
    type Asset = Dialog;
    type Settings = ();
    type Error = LoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let s = std::str::from_utf8(&bytes)?;
            let toml = toml::from_str(s)?;
            Ok(deser::from_toml(toml))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}

impl From<String> for NodeName {
    fn from(s: String) -> Self {
        match s.as_str() {
            "_end_dialog" => NodeName::EndDialog,
            "_emerge" => NodeName::Emerge,
            _ => NodeName::Explicit(s),
        }
    }
}
