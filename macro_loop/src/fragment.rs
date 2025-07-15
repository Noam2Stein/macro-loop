use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, Token,
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    token::{Brace, Paren},
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

    #[allow(private_interfaces)]
    #[peek(Paren, name = "an expe")]
    Expr(FragmentExpr),

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
    #[call(Punctuated::parse_separated_nonempty)]
    segments: Punctuated<FragmentForSegment, Token![,]>,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: TokenStream,
}

#[derive(Clone, Parse)]
struct FragmentForSegment {
    pat: Pattern,
    _in_token: Token![in],
    items: Expr,
}

#[derive(Clone, Parse)]
struct FragmentLet {
    _let_token: Token![let],
    pat: Pattern,
    _eq_token: Token![=],
    value: Expr,
    _semi_token: Token![;],
}

#[derive(Clone, Parse)]
struct FragmentExpr {
    #[paren]
    _parens: Paren,
    #[inside(_parens)]
    expr: Expr,
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
            Self::Expr(self_) => self_.apply(names, tokens),
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
        let lists = self
            .segments
            .iter()
            .map(|segment| Value::from_expr(segment.items.clone(), names.clone()))
            .collect::<syn::Result<Vec<Value>>>()?;

        let lists = lists
            .into_iter()
            .map(|items| {
                if let Value::List(items) = items {
                    Ok(items.items)
                } else {
                    Err(Error::new_spanned(&items, "expected a list"))
                }
            })
            .collect::<syn::Result<Vec<Vec<Value>>>>()?;

        for values in cartesian_product(lists) {
            let mut names = names.clone();

            for (pat, value) in self.segments.iter().map(|seg| &seg.pat).zip(values) {
                pat.insert_to_names(value, &mut names)?;
            }

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

impl ApplyFragment for FragmentExpr {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = Value::from_expr(self.expr, names.clone())?;

        value.to_tokens(tokens);

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

fn cartesian_product<T: Clone>(input: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![vec![]];

    for pool in input {
        let mut new_result = Vec::new();
        for combination in &result {
            for item in &pool {
                let mut new_combination = combination.clone();
                new_combination.push(item.clone());
                new_result.push(new_combination);
            }
        }
        result = new_result;
    }

    result
}
