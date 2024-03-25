//! Namespace represents a unique toml file in the `assets/dialogs` directory.

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
            EnterTheApartmentElevator => "enter_the_apartment_elevator.toml",
            BoltIsMean => "bolt_is_mean.toml",
            MarieBlabbering => "marie_blabbering.toml",
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
        Namespace::from(path["dialogs/".len()..].to_string())
    }
}

impl From<String> for Namespace {
    fn from(file_path: String) -> Self {
        assert!(file_path.ends_with(".toml"));
        assert!(!file_path.starts_with("dialogs/"));
        Namespace { file_path }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn it_validates_dialogs() {
        for namespace in TypedNamespace::iter() {
            println!("Validating {namespace:?}");

            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let path =
                format!("{manifest}/../../main_game/assets/dialogs/{dialog}");
            let toml = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{path}: {e}"));

            DialogGraph::subgraph_from_raw(dialog.into(), &toml);
        }
    }
}
