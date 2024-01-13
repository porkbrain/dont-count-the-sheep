use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
            pub fn spawn(
                cmd: &mut Commands,
                asset_server: &AssetServer
            ) {
                spawn(cmd, asset_server, Self::sequence());
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
