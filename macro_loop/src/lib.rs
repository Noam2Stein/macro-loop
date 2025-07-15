use proc_macro2::TokenStream;
use syn::parse::{ParseStream, Parser};

mod expr;
mod fragment;
mod fragment_expr;
mod fragment_for;
mod fragment_ident;
mod fragment_if;
mod fragment_let;
mod fragment_name;
mod map;
mod ops;
mod pattern;
mod to_tokens_spanned;
mod value;

#[proc_macro]
pub fn macro_loop(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match macro_loop_.parse2(input.into()) {
        Ok(stream) => stream,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn macro_loop_(parser: ParseStream) -> syn::Result<TokenStream> {
    map::map_tokenstream(parser, std::collections::HashMap::new())
}
