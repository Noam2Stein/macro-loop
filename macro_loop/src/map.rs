use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Token,
    parse::{Parse, ParseStream, Parser},
    parse2,
    token::Brace,
};

use super::{param::*, value::*};

pub fn map_tokenstream(
    input: syn::parse::ParseStream,
    values: &HashMap<String, Value>,
) -> syn::Result<TokenStream> {
    let mut output = TokenStream::new();

    while let Some(token) = input.parse::<Option<TokenTree>>()? {
        let is_after_marker = token_is_marker(&token) && !input.peek(Token![@]);

        if is_after_marker {
            let control_flow = ControlFlow::parse(input)?;

            output.append_all(control_flow.map_body(values));
        } else if let TokenTree::Group(group) = &token {
            let stream_fn = |input: ParseStream| map_tokenstream(input, values);
            let stream = stream_fn.parse2(group.stream())?;

            output.append(Group::new(group.delimiter(), stream));
        } else if let TokenTree::Ident(ident) = &token {
            if let Some(value) = values.get(&ident.to_string()) {
                output.append_all(value.to_token_stream());
            } else {
                output.append(token);
            }
        } else {
            output.append(token);
        }
    }

    Ok(output)
}

#[derive(Clone, Parse)]
enum ControlFlow {
    #[peek(Token![for], name = "for loop")]
    For(For),
}

#[derive(Clone, Parse)]
struct For {
    _for_token: Token![for],
    param: Param,
    _in_token: Token![in],
    iter: syn::Expr,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: TokenStream,
}

fn token_is_marker(token: &TokenTree) -> bool {
    if let TokenTree::Punct(punct) = token {
        punct.as_char() == '@'
    } else {
        false
    }
}

impl ControlFlow {
    fn map_body(&self, values: &HashMap<String, Value>) -> syn::Result<TokenStream> {
        match self {
            Self::For(flow) => flow.map_body(values),
        }
    }
}

impl For {
    fn map_body(&self, values: &HashMap<String, Value>) -> syn::Result<TokenStream> {
        let iter = apply_values(&self.iter, values)?;

        let items = if let Value::List(list) = iter {
            list.items
        } else {
            return Err(Error::new_spanned(&iter, "expected a list"));
        };

        items
            .into_iter()
            .map(|item| {
                let mut values = values.clone();

                self.param.insert_values(item, &mut values)?;

                let map_body = |input: ParseStream| map_tokenstream(input, &values);

                map_body.parse2(self.body.clone())
            })
            .collect()
    }
}

fn apply_values(expr: &syn::Expr, values: &HashMap<String, Value>) -> syn::Result<Value> {
    let stream = expr
        .to_token_stream()
        .into_iter()
        .map(|token| {
            if let TokenTree::Ident(ident) = &token {
                if let Some(value) = values.get(&ident.to_string()) {
                    value.to_token_stream()
                } else {
                    TokenStream::from_iter([token])
                }
            } else {
                TokenStream::from_iter([token])
            }
        })
        .collect::<TokenStream>();

    parse2(stream)
}
