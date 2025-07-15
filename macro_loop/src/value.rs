use std::collections::HashMap;

use proc_macro2::{Group, Literal, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, Lit, LitBool, LitStr, Token,
    parse::{ParseStream, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::map::map_tokenstream;

use super::{expr::*, ops::*, to_tokens_spanned::*};

#[derive(Clone)]
pub enum Value {
    Lit(Lit),
    Ident(Ident),
    List(ValueList),
}

#[derive(Clone)]
pub struct ValueList {
    pub span: Span,
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

            Expr::Name(_) => unreachable!(),

            Expr::Bin(ExprBin { lhs, op, rhs }) => {
                let lhs = Value::from_expr(*lhs, names.clone())?;
                let rhs = Value::from_expr(*rhs, names.clone())?;

                match (lhs, rhs) {
                    (Self::Lit(Lit::Int(lhs)), Self::Lit(Lit::Int(rhs))) => Self::int_bin_op(
                        lhs.base10_parse::<u128>()?,
                        op,
                        rhs.base10_parse::<u128>()?,
                    )?,

                    (Self::Lit(Lit::Float(lhs)), Self::Lit(Lit::Float(rhs))) => Self::float_bin_op(
                        lhs.base10_parse::<f64>()?,
                        op,
                        rhs.base10_parse::<f64>()?,
                    )?,

                    (Self::Lit(Lit::Bool(lhs)), Self::Lit(Lit::Bool(rhs))) => {
                        Value::Lit(Lit::Bool(LitBool::new(
                            Self::bool_bin_op(lhs.value, op, rhs.value)?,
                            op.span(),
                        )))
                    }

                    (Self::Lit(Lit::Str(lhs)), Self::Lit(Lit::Str(rhs))) => {
                        Value::Lit(Lit::Str(LitStr::new(
                            &Self::str_bin_op(&lhs.value(), op, &rhs.value())?,
                            op.span(),
                        )))
                    }

                    (Self::Ident(lhs), Self::Ident(rhs)) => Value::Ident(Ident::new(
                        &Self::str_bin_op(&lhs.to_string(), op, &rhs.to_string())?,
                        op.span(),
                    )),

                    _ => return Err(Error::new_spanned(op, "invalid operation")),
                }
            }

            Expr::Un(ExprUn { op, base }) => {
                let base = Value::from_expr(*base, names)?;

                match base {
                    _ => return Err(Error::new_spanned(op, "invalid operation")),
                }
            }
        })
    }

    fn int_bin_op(lhs: u128, op: BinOp, rhs: u128) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Add(_) => int(lhs + rhs),
            BinOp::Sub(_) => int(lhs - rhs),
            BinOp::Mul(_) => int(lhs * rhs),
            BinOp::Div(_) => int(lhs / rhs),
            BinOp::Rem(_) => int(lhs % rhs),

            BinOp::BitAnd(_) => int(lhs & rhs),
            BinOp::BitOr(_) => int(lhs | rhs),
            BinOp::Shl(_) => int(lhs << rhs),
            BinOp::Shr(_) => int(lhs >> rhs),

            BinOp::Range(op) => Self::List(ValueList {
                span: op.span(),
                items: (lhs..rhs).map(|i| int(i)).collect(),
            }),
            BinOp::RangeInclusive(op) => Self::List(ValueList {
                span: op.span(),
                items: (lhs..=rhs).map(|i| int(i)).collect(),
            }),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn float_bin_op(lhs: f64, op: BinOp, rhs: f64) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Add(_) => float(lhs + rhs),
            BinOp::Sub(_) => float(lhs - rhs),
            BinOp::Mul(_) => float(lhs * rhs),
            BinOp::Div(_) => float(lhs / rhs),
            BinOp::Rem(_) => float(lhs % rhs),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn bool_bin_op(lhs: bool, op: BinOp, rhs: bool) -> syn::Result<bool> {
        Ok(match op {
            BinOp::BitAnd(_) => lhs & rhs,
            BinOp::BitOr(_) => lhs | rhs,
            BinOp::BitXor(_) => lhs ^ rhs,

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn str_bin_op(lhs: &str, op: BinOp, rhs: &str) -> syn::Result<String> {
        Ok(match op {
            BinOp::Add(_) => lhs.to_string() + rhs,

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }
}

fn int(value: u128) -> Value {
    Value::Lit(
        parse2(TokenStream::from_iter([TokenTree::Literal(
            Literal::u128_unsuffixed(value),
        )]))
        .unwrap(),
    )
}

fn float(value: f64) -> Value {
    Value::Lit(
        parse2(TokenStream::from_iter([TokenTree::Literal(
            Literal::f64_unsuffixed(value),
        )]))
        .unwrap(),
    )
}
