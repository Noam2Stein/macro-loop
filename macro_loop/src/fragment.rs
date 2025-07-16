use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::{
    Ident, Token,
    token::{Bracket, Paren},
};

use super::{
    fragment_concat::*, fragment_expr::*, fragment_for::*, fragment_if::*, fragment_let::*,
    fragment_name::*, value::*,
};

#[derive(Clone, Parse)]
pub enum Fragment {
    #[allow(private_interfaces)]
    #[peek(Token![for], name = "for")]
    For(FragmentFor),

    #[allow(private_interfaces)]
    #[peek(Token![if], name = "if")]
    If(FragmentIf),

    #[allow(private_interfaces)]
    #[peek(Token![let], name = "let")]
    Let(FragmentLet),

    #[allow(private_interfaces)]
    #[peek(Paren, name = "`()`")]
    Expr(FragmentExpr),

    #[allow(private_interfaces)]
    #[peek(Bracket, name = "`[]`")]
    Ident(FragmentConcat),

    #[peek(Ident, name = "a name")]
    Name(FragmentName),
}

pub trait ApplyFragment {
    fn apply(self, names: &mut HashMap<String, Value>, tokens: &mut TokenStream)
    -> syn::Result<()>;
}

impl ApplyFragment for Fragment {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        match self {
            Self::For(self_) => self_.apply(names, tokens),
            Self::If(self_) => self_.apply(names, tokens),
            Self::Let(self_) => self_.apply(names, tokens),
            Self::Expr(self_) => self_.apply(names, tokens),
            Self::Ident(self_) => self_.apply(names, tokens),
            Self::Name(self_) => self_.apply(names, tokens),
        }
    }
}
