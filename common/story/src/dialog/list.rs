//! Namespace represents a unique toml file in the `assets/dialogs` directory.
//! Namespace does not contain the `dialogs/` prefix nor the `.toml` extension.

use bevy::{
    asset::{AssetPath, Handle},
    reflect::Reflect,
};

use super::DialogGraph;

/// References the dialog in some way.
#[derive(Clone)]
pub enum DialogRef {
    /// Most generic reference.
    /// Path to the asset file.
    Namespace(Namespace),
    /// Reference to a dialog that is used somewhere in the codebase and
    /// therefore has a name.
    TypedNamespace(TypedNamespace),
    /// Strong handle to a dialog already loaded in assets.
    Handle(Handle<DialogGraph>),
}

/// Namespace represents a dialog toml file with relative path from the root
/// of the dialog directory.
/// Each dialog file has a unique name.
#[derive(PartialEq, Eq, Debug, Clone, Hash, Reflect)]
pub struct Namespace {
    /// This can be a file path or a runtime created dialog name.
    unique_name: String,
}

/// Typed dialogs are either files or runtime created dialogs.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, strum::EnumIter)]
pub enum TypedNamespace {
    BoltIsMean,
    MarieBlabbering,
    MrGoodWater,

    // --------------------------------------------------------------
    //
    //
    // These don't reference actual dialog files, but runtime created dialogs
    //
    //
    // --------------------------------------------------------------
    /// This is a special dialog that is created at runtime when the player
    /// enters the elevator.
    InElevator,
}

impl AsRef<str> for TypedNamespace {
    fn as_ref(&self) -> &str {
        use TypedNamespace::*;
        match self {
            MrGoodWater => "mr_good_water",
            BoltIsMean => "bolt_is_mean",
            MarieBlabbering => "marie_blabbering",
            InElevator => "in_elevator",
        }
    }
}

impl std::fmt::Display for TypedNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl From<TypedNamespace> for Namespace {
    fn from(typed: TypedNamespace) -> Self {
        typed.as_ref().to_string().into()
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.unique_name)
    }
}

impl From<&AssetPath<'static>> for Namespace {
    fn from(asset_path: &AssetPath<'static>) -> Self {
        let path = asset_path.to_string();
        assert!(path.starts_with("dialogs/"));
        assert!(path.ends_with(".toml"));
        Namespace::from(
            path["dialogs/".len()..(path.len() - ".toml".len())].to_string(),
        )
    }
}

impl From<String> for Namespace {
    fn from(file_path: String) -> Self {
        Namespace {
            unique_name: file_path
                .trim_end_matches(".toml")
                .trim_start_matches("dialogs/")
                .to_string(),
        }
    }
}

impl From<Namespace> for DialogRef {
    fn from(namespace: Namespace) -> Self {
        DialogRef::Namespace(namespace)
    }
}

impl From<TypedNamespace> for DialogRef {
    fn from(typed: TypedNamespace) -> Self {
        DialogRef::TypedNamespace(typed)
    }
}

impl From<Handle<DialogGraph>> for DialogRef {
    fn from(handle: Handle<DialogGraph>) -> Self {
        DialogRef::Handle(handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_validates_all_dialog_assets() {
        // load all toml files in the dialog directory

        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = format!("{manifest}/../../main_game/assets/dialogs");
        let paths = std::fs::read_dir(&path)
            .unwrap_or_else(|e| panic!("{path}: {e}"))
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().unwrap_or_default() == "toml");

        for path in paths {
            let toml = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{path:?}: {e}"));

            let namespace = Namespace::from(path.to_string_lossy().to_string());
            DialogGraph::subgraph_from_raw(namespace, &toml);
        }
    }
}
