mod embed_dialogs;

use proc_macro::TokenStream;

#[proc_macro]
pub fn embed_dialogs(_: TokenStream) -> TokenStream {
    embed_dialogs::expand()
        .to_string()
        .parse()
        .expect("Failed to create embedded dialogs instance")
}
