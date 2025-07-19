use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

use super::*;

#[derive(Clone)]
pub enum ValueRef<'v> {
    Ref(&'v Value<'v>),
    Owned(Value<'v>),
}

impl<'v> Deref for ValueRef<'v> {
    type Target = Value<'v>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(value) => value,
            Self::Ref(value) => value,
        }
    }
}

impl<'v> ToTokens for ValueRef<'v> {
    fn to_token_stream(&self) -> TokenStream {
        Value::to_token_stream(&self)
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        Value::to_tokens(&self, tokens);
    }
}

impl<'v> ToTokensSpanned for ValueRef<'v> {
    fn to_token_stream_spanned(&self, span: Span) -> TokenStream {
        Value::to_token_stream_spanned(&self, span)
    }

    fn to_tokens_spanned(&self, span: Span, tokens: &mut TokenStream) {
        Value::to_tokens_spanned(&self, span, tokens);
    }
}
