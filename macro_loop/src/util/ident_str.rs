use proc_macro2::Span;
use quote::ToTokens;
use syn::{Ident, parse::Parse};

use super::*;

#[derive(Debug, Clone)]
pub struct IdentStr {
    str: Box<str>,
    span: Span,
}

impl IdentStr {
    pub fn new(str: impl Into<Box<str>>, span: Span) -> Self {
        Self {
            str: str.into(),
            span,
        }
    }

    pub fn str(&self) -> &str {
        &self.str
    }

    pub fn set_span(&mut self, value: Span) {
        self.span = value;
    }
}

impl Parse for IdentStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        Ok(Self {
            str: ident.to_string().into_boxed_str(),
            span: ident.span(),
        })
    }
}

impl ToTokens for IdentStr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        Ident::new(&self.str, self.span).to_tokens(tokens);
    }
}

impl Spanned for IdentStr {
    fn span(&self) -> Span {
        self.span
    }
}
