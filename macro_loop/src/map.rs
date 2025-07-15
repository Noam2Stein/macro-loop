use std::collections::HashMap;

use proc_macro2::{Group, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{
    Token,
    parse::{Parse, ParseStream, Parser},
};

use super::{frag::*, value::*};

pub fn map_tokenstream(
    input: syn::parse::ParseStream,
    mut names: HashMap<String, Value>,
) -> syn::Result<TokenStream> {
    let mut output = TokenStream::new();

    while let Some(token) = input.parse::<Option<TokenTree>>()? {
        let is_after_marker = token_is_marker(&token) && !input.peek(Token![@]);

        if is_after_marker {
            let frag = Fragment::parse(input)?;

            frag.apply(&mut names, &mut output)?;
        } else if let TokenTree::Group(group) = &token {
            let map_fn = |input: ParseStream| map_tokenstream(input, names.clone());
            let group_stream = map_fn.parse2(group.stream())?;

            output.append(Group::new(group.delimiter(), group_stream));
        } else {
            output.append(token);
        }
    }

    Ok(output)
}

fn token_is_marker(token: &TokenTree) -> bool {
    if let TokenTree::Punct(punct) = token {
        punct.as_char() == '@'
    } else {
        false
    }
}
