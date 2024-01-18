use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Implements `spawn` for the type.
#[proc_macro_derive(Portrait)]
pub fn derive_portrait(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let expanded = quote! {
        impl #name {
            /// Spawns the dialog into the world.
            pub fn spawn(
                cmd: &mut bevy::prelude::Commands,
                asset_server: &bevy::prelude::AssetServer,
                global_store: &common_store::GlobalStore,
            ) {
                crate::portrait_dialog::spawn(
                    cmd,
                    asset_server,
                    global_store,
                    Self::sequence(),
                );
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
