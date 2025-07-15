use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Error, Ident, Token,
    parse::{ParseStream, Parser},
    token::Brace,
};

use crate::{map::map_tokenstream, to_tokens_spanned::ToTokensSpanned};

use super::{expr::*, pattern::*, value::*};

#[derive(Clone, Parse)]
pub enum Fragment {
    #[allow(private_interfaces)]
    #[peek(Token![for], name = "a for-loop")]
    For(FragmentFor),

    #[allow(private_interfaces)]
    #[peek(Token![let], name = "let")]
    Let(FragmentLet),

    #[peek(Ident, name = "a name")]
    Name(Ident),
}

pub trait ApplyFragment {
    fn apply(self, names: &mut HashMap<String, Value>, tokens: &mut TokenStream)
    -> syn::Result<()>;
}

#[derive(Clone, Parse)]
struct FragmentFor {
    _for_token: Token![for],
    pat: Pattern,
    _in_token: Token![in],
    items: Expr,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: TokenStream,
}

#[derive(Clone, Parse)]
struct FragmentLet {
    _let_token: Token![let],
    pat: Pattern,
    _eq_token: Token![=],
    value: Expr,
    _semi_token: Token![;],
}

impl ApplyFragment for Fragment {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        match self {
            Self::For(self_) => self_.apply(names, tokens),
            Self::Let(self_) => self_.apply(names, tokens),
            Self::Name(self_) => self_.apply(names, tokens),
        }
    }
}

impl ApplyFragment for FragmentFor {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let items = Value::from_expr(self.items, names.clone())?;

        let items = if let Value::List(items) = items {
            items.items
        } else {
            return Err(Error::new_spanned(&items, "expected a list"));
        };

        for item in items {
            let mut names = names.clone();

            self.pat.insert_to_names(item, &mut names)?;

            let map_fn = |input: ParseStream| map_tokenstream(input, names);
            tokens.append_all(map_fn.parse2(self.body.clone())?);
        }

        Ok(())
    }
}

impl ApplyFragment for FragmentLet {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        _tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = Value::from_expr(self.value, names.clone())?;

        self.pat.insert_to_names(value, names)?;

        Ok(())
    }
}

impl ApplyFragment for Ident {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = match names.get(&self.to_string()) {
            Some(value) => value,
            None => return Err(Error::new_spanned(&self, format!("can't find {self}"))),
        };

        value.to_tokens_spanned(self.span(), tokens);

        Ok(())
    }
}
