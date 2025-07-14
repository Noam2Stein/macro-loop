use proc_macro2::TokenStream;
use syn::parse::{ParseStream, Parser};

mod expr;
mod map;
mod op;
mod param;
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
    map::map_tokenstream(parser, &std::collections::HashMap::new())
}
