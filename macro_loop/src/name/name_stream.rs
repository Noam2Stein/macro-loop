use std::mem::take;

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Token,
    parse::{Parse, ParseStream, Parser},
    token::{Brace, Bracket, Paren},
};

use super::*;

pub struct NameStream {
    segs: Vec<NameStreamSegment>,
}

enum NameStreamSegment {
    TokenStream(TokenStream),
    Group(NameStreamGroup),
    Fragment(Frag),
}

struct NameStreamGroup {
    span: Span,
    delim: Delimiter,
    stream: Box<NameStream>,
}

impl Parse for NameStream {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut output = Self { segs: Vec::new() };

        let mut tokenstream = TokenStream::new();

        while !input.is_empty() {
            if input.peek(Token![@]) {
                let _ = input.parse::<Token![@]>().unwrap();

                output
                    .segs
                    .push(NameStreamSegment::TokenStream(take(&mut tokenstream)));

                let fragment = input.parse::<Frag>()?;

                output.segs.push(NameStreamSegment::Fragment(fragment));
            } else if input.peek(Brace) || input.peek(Paren) || input.peek(Bracket) {
                let group = input.parse::<Group>().unwrap();

                output
                    .segs
                    .push(NameStreamSegment::TokenStream(take(&mut tokenstream)));

                output.segs.push(NameStreamSegment::Group(NameStreamGroup {
                    span: group.span(),
                    delim: group.delimiter(),
                    stream: Box::new(NameStream::parse.parse2(group.stream())?),
                }));
            } else {
                tokenstream.append(input.parse::<TokenTree>()?);
            }
        }

        output
            .segs
            .push(NameStreamSegment::TokenStream(take(&mut tokenstream)));

        Ok(output)
    }
}

impl NameStream {
    pub fn resolve(&self, namespace: &Namespace) -> syn::Result<TokenStream> {
        let mut output = TokenStream::new();

        let mut namespace = namespace.fork();

        for seg in &self.segs {
            match seg {
                NameStreamSegment::TokenStream(stream) => stream.to_tokens(&mut output),

                NameStreamSegment::Group(group) => {
                    let mut token = Group::new(group.delim, group.stream.resolve(&namespace)?);
                    token.set_span(group.span);

                    output.append(token);
                }

                NameStreamSegment::Fragment(frag) => frag.apply(&mut namespace, &mut output)?,
            }

            namespace.flush();
        }

        Ok(output)
    }
}
