#![doc = include_str!("../../README.md")]
#![deny(missing_docs)]

pub mod godot;
mod lex;
mod parse;
pub mod value;

use lex::lex;
use parse::parse;

/// Parses Godot's .tscn file.
pub fn from_tscn(tscn: &str) -> miette::Result<godot::Scene> {
    lex(tscn).and_then(|tokens| parse(tscn, tokens))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{lex, parse};

    #[test]
    fn it_lexes_and_parses_tscn() -> miette::Result<()> {
        let workspace_root =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect(
                "Failed to get CARGO_MANIFEST_DIR environment variable",
            ));
        let scenes_dir = if workspace_root.ends_with("rscn") {
            // we are running this test from the rscn directory
            format!("{}/../main_game/assets/scenes", workspace_root.display())
        } else {
            // we are running this test from the workspace root
            format!("{}/main_game/assets/scenes", workspace_root.display())
        };
        let dir_iter = std::fs::read_dir(&scenes_dir).unwrap_or_else(|err| {
            panic!("Failed to read directory '{scenes_dir}' with .tscn files: {err}");
        });

        for entry in dir_iter {
            let entry = entry
                .expect("Failed to read entry in directory with .tscn files");
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "tscn") {
                continue;
            }

            let tscn = std::fs::read_to_string(&path).unwrap_or_else(|err| {
                panic!("Failed to read .tscn file at {path:?}: {err}");
            });

            let tokens = lex(&tscn)?;
            assert!(!tokens.is_empty(), "Empty .tscn file at {path:?}");

            parse(&tscn, tokens)?;
        }

        Ok(())
    }
}
