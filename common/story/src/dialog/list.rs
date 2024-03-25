//! Namespace represents a unique toml file in the `assets/dialogs` directory.
//! Namespace does not contain the `dialogs/` prefix nor the `.toml` extension.

use bevy::{asset::AssetPath, reflect::Reflect};

/// Namespace represents a dialog toml file with relative path from the root
/// of the dialog directory.
/// Each dialog file has a unique name.
#[derive(PartialEq, Eq, Debug, Clone, Hash, Reflect)]
pub struct Namespace {
    file_path: String,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, strum::EnumIter)]
pub enum TypedNamespace {
    EnterTheApartmentElevator,
    BoltIsMean,
    MarieBlabbering,
}

impl AsRef<str> for TypedNamespace {
    fn as_ref(&self) -> &str {
        use TypedNamespace::*;
        match self {
            EnterTheApartmentElevator => "enter_the_apartment_elevator",
            BoltIsMean => "bolt_is_mean",
            MarieBlabbering => "marie_blabbering",
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
        write!(f, "{}", self.file_path)
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
            file_path: file_path
                .trim_end_matches(".toml")
                .trim_start_matches("dialogs/")
                .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;
    use crate::dialog::DialogGraph;

    #[test]
    fn it_validates_typed_dialogs() {
        for namespace in TypedNamespace::iter() {
            println!("Validating {namespace:?}");

            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = format!(
                "{manifest}/../../main_game/assets/dialogs/{namespace}.toml"
            );
            let toml = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{path}: {e}"));

            DialogGraph::subgraph_from_raw(namespace.into(), &toml);
        }
    }

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
