use std::{
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use quote::{format_ident, quote};

pub fn expand() -> proc_macro2::TokenStream {
    let asset_dir = dialog_directory();
    let embedded_dialog_variants = dialog_variants_from_path(&asset_dir);

    quote! {
        #[allow(missing_docs)]
        #[derive(
            PartialEq, Eq, Debug, Clone, Copy, Hash,
            bevy::prelude::Reflect,
            strum::EnumIter, strum::EnumMessage,
        )]
        pub enum DialogRoot {
            #( #embedded_dialog_variants, )*
        }
    }
}

fn dialog_directory() -> PathBuf {
    let cargo_toml_directory =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    Path::new(&cargo_toml_directory)
        .join("..")
        .join("..")
        .join("dialogs")
        .canonicalize()
        .expect("Failed to canonicalize dialog asset directory")
}

fn dialog_variants_from_path(path: &Path) -> Vec<proc_macro2::TokenStream> {
    dialog_toml_files(path)
        .map(|path| dialog_literal_from_path(&path))
        .collect()
}

fn dialog_toml_files(path: &'_ Path) -> impl Iterator<Item = PathBuf> + '_ {
    path.read_dir()
        .expect("Cannot read dialog assets dir")
        .filter_map(|entry_res| {
            let entry = entry_res.expect("Failed to read directory entry");
            if entry
                .metadata()
                .expect("Failed to read entry metadata")
                .is_file()
                && entry.file_name().to_string_lossy().ends_with(".toml")
            {
                Some(entry.path())
            } else {
                None
            }
        })
}

fn dialog_literal_from_path(path: &Path) -> proc_macro2::TokenStream {
    let enum_variant_str = path
        .file_stem()
        .unwrap_or_else(|| panic!("Can't get file name from path '{path:?}'"))
        .to_string_lossy()
        .to_case(Case::Pascal);
    let enum_variant = format_ident!("{enum_variant_str}");

    let file_path = path.to_str().expect("Failed to convert path to string");

    let toml = {
        let mut s = String::new();
        let mut file = File::open(path).expect("Failed to open dialog file");
        file.read_to_string(&mut s)
            .expect("Failed to read dialog file");

        s
    };

    {
        // validates the toml
        toml::from_str::<toml::Value>(&toml).unwrap_or_else(|e| {
            panic!("Invalid TOML dialog file {path:?}: {e}")
        });
    }

    quote!(
        #[strum(message = #file_path, detailed_message = #toml)]
        #enum_variant
    )
}
