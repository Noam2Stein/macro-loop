use std::collections::HashMap;

use proc_macro2::{Group, Span, TokenStream, TokenTree, extra::DelimSpan};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, Lit, Token,
    parse::{ParseStream, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::map::map_tokenstream;

use super::{expr::*, op::*, to_tokens_spanned::*};

#[derive(Clone)]
pub enum Value {
    Lit(Lit),
    Ident(Ident),
    List(ValueList),
}

#[derive(Clone)]
pub struct ValueList {
    pub span: DelimSpan,
    pub items: Vec<Value>,
}

impl ToTokensSpanned for Value {
    fn to_tokens_spanned(&self, span: Span, tokens: &mut TokenStream) {
        match self {
            Self::List(list) => list.to_tokens_spanned(span, tokens),

            Self::Ident(ident) => {
                let mut ident = ident.clone();
                ident.set_span(span);

                ident.to_tokens(tokens);
            }

            Self::Lit(lit) => {
                let mut lit = lit.clone();
                lit.set_span(span);

                lit.to_tokens(tokens);
            }
        }
    }
}

impl ToTokensSpanned for ValueList {
    fn to_tokens_spanned(&self, span: Span, tokens: &mut TokenStream) {
        let group_stream = Punctuated::<_, Token![,]>::from_iter(
            self.items
                .iter()
                .map(|item| item.to_token_stream_spanned(span)),
        )
        .to_token_stream();

        let mut group = Group::new(proc_macro2::Delimiter::Bracket, group_stream);
        group.set_span(self.span.span());

        tokens.append(TokenTree::Group(group));
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::List(list) => list.to_tokens(tokens),
        }
    }
}

impl ToTokens for ValueList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let group_stream = Punctuated::<_, Token![,]>::from_iter(&self.items).to_token_stream();

        let mut group = Group::new(proc_macro2::Delimiter::Bracket, group_stream);
        group.set_span(self.span.span());

        tokens.append(TokenTree::Group(group));
    }
}

impl Value {
    pub fn from_expr(expr: Expr, names: HashMap<String, Value>) -> syn::Result<Self> {
        let map_fn = |input: ParseStream| map_tokenstream(input, names.clone());
        let expr = parse2::<Expr>(map_fn.parse2(expr.to_token_stream())?)?;

        Ok(match expr {
            Expr::Lit(lit) => Self::Lit(lit),
            Expr::Ident(ident) => Self::Ident(ident),

            Expr::List(list) => Self::List(ValueList {
                span: list.span,
                items: list
                    .items
                    .into_iter()
                    .map(|item| Self::from_expr(item, names.clone()))
                    .collect::<syn::Result<_>>()?,
            }),

            Expr::Name(name) => unreachable!(),

            Expr::Bin(ExprBin { lhs, op, rhs }) => {
                let lhs = Value::from_expr(*lhs, names.clone())?;
                let rhs = Value::from_expr(*rhs, names.clone())?;

                match op {
                    BinOp::Add(op) => Self::add(lhs, op, rhs)?,
                    BinOp::Sub(op) => Self::sub(lhs, op, rhs)?,
                    BinOp::Mul(op) => Self::mul(lhs, op, rhs)?,
                    BinOp::Div(op) => Self::div(lhs, op, rhs)?,
                    BinOp::Rem(op) => Self::rem(lhs, op, rhs)?,

                    BinOp::And(op) => Self::and(lhs, op, rhs)?,
                    BinOp::Or(op) => Self::or(lhs, op, rhs)?,
                    BinOp::Xor(op) => Self::xor(lhs, op, rhs)?,
                    BinOp::Shl(op) => Self::shl(lhs, op, rhs)?,
                    BinOp::Shr(op) => Self::shr(lhs, op, rhs)?,

                    BinOp::Range(op) => Self::range(lhs, op, rhs)?,
                    BinOp::RangeInclusive(op) => Self::range_inclusive(lhs, op, rhs)?,
                }
            }

            Expr::Un(ExprUn { op, base }) => {
                let base = Value::from_expr(*base, names)?;

                match op {
                    UnOp::Neg(op) => Self::neg(op, base)?,
                    UnOp::Not(op) => Self::not(op, base)?,
                }
            }
        })
    }

    fn add(lhs: Self, op: Token![+], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid add")),
        })
    }

    fn sub(lhs: Self, op: Token![-], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid sub")),
        })
    }

    fn mul(lhs: Self, op: Token![*], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid mul")),
        })
    }

    fn div(lhs: Self, op: Token![/], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid div")),
        })
    }

    fn rem(lhs: Self, op: Token![%], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid rem")),
        })
    }

    fn and(lhs: Self, op: Token![&], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid and")),
        })
    }

    fn or(lhs: Self, op: Token![|], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid or")),
        })
    }

    fn xor(lhs: Self, op: Token![^], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid xor")),
        })
    }

    fn shl(lhs: Self, op: Token![<<], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid shl")),
        })
    }

    fn shr(lhs: Self, op: Token![>>], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid shr")),
        })
    }

    fn range(lhs: Self, op: Token![..], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid range")),
        })
    }

    fn range_inclusive(lhs: Self, op: Token![..=], rhs: Self) -> syn::Result<Self> {
        Ok(match (lhs, rhs) {
            _ => return Err(Error::new_spanned(op, "invalid range")),
        })
    }

    fn neg(op: Token![-], base: Self) -> syn::Result<Self> {
        Ok(match base {
            _ => return Err(Error::new_spanned(op, "invalid neg")),
        })
    }

    fn not(op: Token![!], base: Self) -> syn::Result<Self> {
        Ok(match base {
            _ => return Err(Error::new_spanned(op, "invalid not")),
        })
    }
}
