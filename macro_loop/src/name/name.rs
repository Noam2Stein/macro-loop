use std::{hash::Hash, ops::Deref};

use derive_more::Display;
use proc_macro2::Span;
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

#[derive(Debug, Clone, Display)]
#[display("id")]
pub struct Name {
    id: NameId,
    span: Span,
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Display)]
pub struct NameId {
    inner: Box<str>,
}

impl Name {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn id(&self) -> &NameId {
        &self.id
    }
}

impl Deref for Name {
    type Target = NameId;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl Parse for Name {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        Ok(Self {
            id: NameId {
                inner: ident.to_string().into_boxed_str(),
            },
            span: ident.span(),
        })
    }
}
