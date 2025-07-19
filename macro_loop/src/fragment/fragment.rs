use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Ident, Token,
    token::{Bracket, Paren},
};

use super::*;

#[derive(Parse)]
pub enum Frag {
    #[allow(private_interfaces)]
    #[peek(Token![for], name = "for")]
    For(FragFor),

    #[allow(private_interfaces)]
    #[peek(Token![if], name = "if")]
    If(FragIf),

    #[allow(private_interfaces)]
    #[peek(Token![let], name = "let")]
    Let(FragLet),

    #[allow(private_interfaces)]
    #[peek(Paren, name = "`()`")]
    Expr(FragExpr),

    #[allow(private_interfaces)]
    #[peek(Bracket, name = "`[]`")]
    Ident(FragConcat),

    #[peek(Ident, name = "a name")]
    Name(FragName),

    #[peek(Token![@], name = "`@`")]
    Cancel(Token![@]),
}

pub trait ApplyFragment {
    fn apply<'s: 'v, 'v>(
        &'s self,
        namespace: &mut Namespace<'v, 'v>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()>;
}

impl ApplyFragment for Frag {
    fn apply<'s: 'v, 'v>(
        &'s self,
        namespace: &mut Namespace<'v, 'v>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        match self {
            Self::For(self_) => self_.apply(namespace, tokens),
            Self::If(self_) => self_.apply(namespace, tokens),
            Self::Let(self_) => self_.apply(namespace, tokens),
            Self::Expr(self_) => self_.apply(namespace, tokens),
            Self::Ident(self_) => self_.apply(namespace, tokens),
            Self::Name(self_) => self_.apply(namespace, tokens),

            Self::Cancel(self_) => Ok(self_.to_tokens(tokens)),
        }
    }
}
