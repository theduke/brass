use proc_macro::TokenStream;

mod view;

/// Construct dom nodes with a convenient helper syntax.
#[proc_macro]
pub fn view(tokens: TokenStream) -> TokenStream {
    view::view(tokens)
}
