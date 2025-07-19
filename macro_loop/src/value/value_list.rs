use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{Error, Token, punctuated::Punctuated, spanned::Spanned};

use super::*;

#[derive(Clone)]
pub struct ValueList<'v> {
    pub span: Span,
    pub items: Vec<ValueRef<'v>>,
}

impl<'v> ToTokens for ValueList<'v> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let group_stream = Punctuated::<_, Token![,]>::from_iter(&self.items).to_token_stream();

        let mut group = Group::new(Delimiter::Bracket, group_stream);
        group.set_span(self.span.span());

        tokens.append(TokenTree::Group(group));
    }
}

impl<'v> ToTokensSpanned for ValueList<'v> {
    fn to_tokens_spanned(&self, span: Span, tokens: &mut TokenStream) {
        let group_stream = Punctuated::<_, Token![,]>::from_iter(
            self.items
                .iter()
                .map(|item| item.to_token_stream_spanned(span)),
        )
        .to_token_stream();

        let mut group = Group::new(Delimiter::Bracket, group_stream);
        group.set_span(self.span.span());

        tokens.append(TokenTree::Group(group));
    }
}

// Index

impl<'v> ValueRef<'v> {
    pub fn index_cloned(&self, idx: usize, span: Span) -> syn::Result<ValueRef<'v>> {
        Ok(match self {
            Self::Owned(Value::List(self_)) => self_.index_cloned(idx, span)?,

            Self::Ref(Value::List(self_)) => ValueRef::Ref(self_.index_ref(idx, span)?),

            _ => return Err(Error::new(span, "non-lists cannot be indexed")),
        })
    }

    #[allow(dead_code)]
    pub fn index_ref(&'v self, idx: usize, span: Span) -> syn::Result<&'v ValueRef<'v>> {
        Ok(match self {
            Self::Owned(Value::List(self_)) | Self::Ref(Value::List(self_)) => {
                self_.index_ref(idx, span)?
            }

            _ => return Err(Error::new(span, "non-lists cannot be indexed")),
        })
    }
}

impl<'v> ValueList<'v> {
    pub fn index_cloned(&self, idx: usize, span: Span) -> syn::Result<ValueRef<'v>> {
        match self.items.get(idx) {
            Some(item) => Ok(item.clone()),
            None => Err(Error::new(span, "index is out of bounds")),
        }
    }

    pub fn index_ref(&'v self, idx: usize, span: Span) -> syn::Result<&'v ValueRef<'v>> {
        match self.items.get(idx) {
            Some(item) => Ok(item),
            None => Err(Error::new(span, "index is out of bounds")),
        }
    }
}
