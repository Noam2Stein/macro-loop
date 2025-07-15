use std::collections::HashMap;

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, LitBool, LitByteStr, LitCStr, LitChar, LitFloat, LitInt, LitStr, Token,
    parse::{ParseStream, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};

use crate::map::map_tokenstream;

use super::{expr::*, ops::*, to_tokens_spanned::*};

#[derive(Clone)]
pub enum Value {
    Bool(LitBool),
    Int(LitInt),
    Float(LitFloat),
    Str(LitStr),
    Char(LitChar),
    CStr(LitCStr),
    ByteStr(LitByteStr),
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
        macro_rules! set_span {
            ($self_:ident) => {{
                let mut self_ = $self_.clone();
                self_.set_span(span);
                self_.to_tokens(tokens);
            }};
        }

        match self {
            Self::List(list) => list.to_tokens_spanned(span, tokens),

            Self::Bool(self_) => set_span!(self_),
            Self::Int(self_) => set_span!(self_),
            Self::Float(self_) => set_span!(self_),
            Self::Str(self_) => set_span!(self_),
            Self::Char(self_) => set_span!(self_),
            Self::CStr(self_) => set_span!(self_),
            Self::ByteStr(self_) => set_span!(self_),
            Self::Ident(self_) => set_span!(self_),
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
            Self::Bool(self_) => self_.to_tokens(tokens),
            Self::Int(self_) => self_.to_tokens(tokens),
            Self::Float(self_) => self_.to_tokens(tokens),
            Self::Str(self_) => self_.to_tokens(tokens),
            Self::Char(self_) => self_.to_tokens(tokens),
            Self::CStr(self_) => self_.to_tokens(tokens),
            Self::ByteStr(self_) => self_.to_tokens(tokens),
            Self::Ident(self_) => self_.to_tokens(tokens),
            Self::List(self_) => self_.to_tokens(tokens),
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
            Expr::Bool(self_) => Self::Bool(self_),
            Expr::Int(self_) => Self::Int(self_),
            Expr::Float(self_) => Self::Float(self_),
            Expr::Str(self_) => Self::Str(self_),
            Expr::Char(self_) => Self::Char(self_),
            Expr::CStr(self_) => Self::CStr(self_),
            Expr::ByteStr(self_) => Self::ByteStr(self_),
            Expr::Ident(self_) => Self::Ident(self_),

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
                    (Self::Bool(lhs), Self::Bool(rhs)) => {
                        Self::bool_bin_op(lhs.value, op, rhs.value)?
                    }

                    (Self::Int(lhs), Self::Int(rhs)) => Self::int_bin_op(
                        lhs.base10_parse::<u128>()?,
                        op,
                        rhs.base10_parse::<u128>()?,
                    )?,

                    (Self::Float(lhs), Self::Float(rhs)) => Self::float_bin_op(
                        lhs.base10_parse::<f64>()?,
                        op,
                        rhs.base10_parse::<f64>()?,
                    )?,

                    (Self::Str(lhs), Self::Str(rhs)) => {
                        Self::str_bin_op(&lhs.value(), op, &rhs.value())?
                    }

                    (Self::Ident(lhs), Self::Ident(rhs)) => {
                        Self::ident_bin_op(&lhs.to_string(), op, &rhs.to_string())?
                    }

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
            BinOp::Add(_) => int(lhs + rhs, op.span()),
            BinOp::Sub(_) => int(lhs - rhs, op.span()),
            BinOp::Mul(_) => int(lhs * rhs, op.span()),
            BinOp::Div(_) => int(lhs / rhs, op.span()),
            BinOp::Rem(_) => int(lhs % rhs, op.span()),

            BinOp::BitAnd(_) => int(lhs & rhs, op.span()),
            BinOp::BitOr(_) => int(lhs | rhs, op.span()),
            BinOp::Shl(_) => int(lhs << rhs, op.span()),
            BinOp::Shr(_) => int(lhs >> rhs, op.span()),

            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            BinOp::Range(op) => Self::List(ValueList {
                span: op.span(),
                items: (lhs..rhs).map(|i| int(i, op.span())).collect(),
            }),
            BinOp::RangeInclusive(op) => Self::List(ValueList {
                span: op.span(),
                items: (lhs..=rhs).map(|i| int(i, op.span())).collect(),
            }),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn float_bin_op(lhs: f64, op: BinOp, rhs: f64) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Add(_) => float(lhs + rhs, op.span()),
            BinOp::Sub(_) => float(lhs - rhs, op.span()),
            BinOp::Mul(_) => float(lhs * rhs, op.span()),
            BinOp::Div(_) => float(lhs / rhs, op.span()),
            BinOp::Rem(_) => float(lhs % rhs, op.span()),

            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn bool_bin_op(lhs: bool, op: BinOp, rhs: bool) -> syn::Result<Self> {
        Ok(match op {
            BinOp::BitAnd(_) | BinOp::LogicalAnd(_) => bool(lhs & rhs, op.span()),
            BinOp::BitOr(_) | BinOp::LogicalOr(_) => bool(lhs | rhs, op.span()),
            BinOp::BitXor(_) => bool(lhs ^ rhs, op.span()),

            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn str_bin_op(lhs: &str, op: BinOp, rhs: &str) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Add(_) => string(lhs.to_string() + rhs, op.span()),

            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn ident_bin_op(lhs: &str, op: BinOp, rhs: &str) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Add(_) => ident(lhs.to_string() + rhs, op.span()),

            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }
}

fn int(value: u128, span: Span) -> Value {
    Value::Int(LitInt::new(&value.to_string(), span))
}

fn float(value: f64, span: Span) -> Value {
    Value::Float(LitFloat::new(&value.to_string(), span))
}

fn bool(value: bool, span: Span) -> Value {
    Value::Bool(LitBool { value, span })
}

fn string(value: impl AsRef<str>, span: Span) -> Value {
    Value::Str(LitStr::new(value.as_ref(), span))
}

fn ident(value: impl AsRef<str>, span: Span) -> Value {
    Value::Ident(Ident::new(value.as_ref(), span))
}
