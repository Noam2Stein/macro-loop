use std::ffi::CStr;

use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, Lit, LitBool, LitByteStr, LitCStr, LitChar, LitFloat, LitInt, LitStr, Token,
    parse::{Parse, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
};

use super::{expr::*, fragment::*, namespace::*, ops::*, to_tokens_spanned::*};

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
            Self::Bool(self_) => set_span!(self_),
            Self::Int(self_) => set_span!(self_),
            Self::Float(self_) => set_span!(self_),
            Self::Str(self_) => set_span!(self_),
            Self::Char(self_) => set_span!(self_),
            Self::CStr(self_) => set_span!(self_),
            Self::ByteStr(self_) => set_span!(self_),
            Self::Ident(self_) => set_span!(self_),

            Self::List(list) => list.to_tokens_spanned(span, tokens),
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
    pub fn from_expr(expr: &Expr, namespace: &Namespace) -> syn::Result<Self> {
        Ok(match expr {
            Expr::Paren(inner) => Self::from_expr(inner, namespace)?,

            Expr::Value(value) => value.clone(),

            Expr::List(list) => Self::List(ValueList {
                span: list.span,
                items: list
                    .items
                    .iter()
                    .map(|item| Self::from_expr(item, namespace))
                    .collect::<syn::Result<_>>()?,
            }),

            Expr::Bin(ExprBin { lhs, op, rhs }) => {
                let lhs = Value::from_expr(lhs, namespace)?;
                let rhs = Value::from_expr(rhs, namespace)?;

                match (lhs, rhs) {
                    (Self::Bool(lhs), Self::Bool(rhs)) => {
                        Self::bool_bin_op(lhs.value, *op, rhs.value)?
                    }

                    (Self::Int(lhs), Self::Int(rhs)) => Self::int_bin_op(
                        lhs.base10_parse::<u128>()?,
                        *op,
                        rhs.base10_parse::<u128>()?,
                    )?,

                    (Self::Float(lhs), Self::Float(rhs)) => Self::float_bin_op(
                        lhs.base10_parse::<f64>()?,
                        *op,
                        rhs.base10_parse::<f64>()?,
                    )?,

                    (Self::Str(lhs), Self::Str(rhs)) => {
                        Self::str_bin_op(&lhs.value(), *op, &rhs.value())?
                    }

                    (Self::Char(lhs), Self::Char(rhs)) => {
                        Self::char_bin_op(lhs.value(), *op, rhs.value())?
                    }

                    (Self::CStr(lhs), Self::CStr(rhs)) => {
                        Self::cstr_bin_op(&lhs.value(), *op, &rhs.value())?
                    }

                    (Self::ByteStr(lhs), Self::ByteStr(rhs)) => {
                        Self::byte_str_bin_op(&lhs.value(), *op, &rhs.value())?
                    }

                    (Self::Ident(lhs), Self::Ident(rhs)) => {
                        Self::ident_bin_op(&lhs.to_string(), *op, &rhs.to_string())?
                    }

                    _ => return Err(Error::new_spanned(op, "invalid operation")),
                }
            }

            Expr::Un(ExprUn { op, base }) => {
                let base = Value::from_expr(base, namespace)?;

                match base {
                    _ => return Err(Error::new_spanned(op, "invalid operation")),
                }
            }

            Expr::Method(expr) => {
                let base = Value::from_expr(&expr.base, namespace)?;
                let inputs = expr
                    .inputs
                    .iter()
                    .map(|input| Value::from_expr(input, namespace))
                    .collect::<syn::Result<_>>()?;

                match expr.method.to_string().as_str() {
                    "enumerate" => Self::enumerate_method(base, expr.method.span(), inputs)?,
                    "index" => Self::index_method(base, expr.method.span(), inputs)?,
                    "min" => Self::min_method(base, expr.method.span(), inputs)?,
                    "max" => Self::max_method(base, expr.method.span(), inputs)?,
                    "clamp" => Self::clamp_method(base, expr.method.span(), inputs)?,

                    _ => return Err(Error::new_spanned(&expr.method, "Unknown method")),
                }
            }

            Expr::Frag(ExprFrag { _at_token: _, frag }) => {
                let mut namespace = namespace.fork();

                let mut output = TokenStream::new();
                frag.apply(&mut namespace, &mut output)?;

                let expr = Expr::parse.parse2(output)?;

                Value::from_expr(&expr, &namespace)?
            }
        })
    }

    pub fn from_lit(lit: Lit) -> syn::Result<Self> {
        Ok(match lit {
            Lit::Bool(lit) => Self::Bool(lit),
            Lit::ByteStr(lit) => Self::ByteStr(lit),
            Lit::CStr(lit) => Self::CStr(lit),
            Lit::Char(lit) => Self::Char(lit),
            Lit::Float(lit) => Self::Float(lit),
            Lit::Int(lit) => Self::Int(lit),
            Lit::Str(lit) => Self::Str(lit),

            Lit::Byte(lit) => Self::Int(LitInt::new(&lit.value().to_string(), lit.span())),

            _ => return Err(Error::new(lit.span(), "unsupported literal")),
        })
    }

    pub fn try_to_string(&self) -> syn::Result<String> {
        Ok(match self {
            Self::Bool(lit) => lit.value.to_string(),
            Self::Int(lit) => lit.base10_parse::<u128>().unwrap().to_string(),
            Self::Str(lit) => lit.value(),
            Self::Char(lit) => lit.value().to_string(),
            Self::CStr(lit) => lit.value().to_str().unwrap().to_string(),
            Self::ByteStr(lit) => String::from_utf8(lit.value()).unwrap(),
            Self::Ident(ident) => ident.to_string(),

            Self::List(list) => list
                .items
                .iter()
                .map(|item| item.try_to_string())
                .collect::<syn::Result<String>>()?,

            _ => {
                return Err(Error::new_spanned(self, "not an identifier value"));
            }
        })
    }

    fn min_method(base: Value, span: Span, mut inputs: Vec<Value>) -> syn::Result<Self> {
        if inputs.len() != 1 {
            return Err(Error::new(span, "expected one argument"));
        }

        let other = inputs.remove(0);

        let other_is_greater = Self::from_expr(
            &Expr::Bin(ExprBin {
                lhs: Box::new(Expr::Value(base.clone())),
                op: BinOp::Gt(Token![>](span)),
                rhs: Box::new(Expr::Value(other.clone())),
            }),
            &Namespace::new(),
        )?;

        let other_is_greater = match other_is_greater {
            Self::Bool(b) => b.value,
            _ => unreachable!(),
        };

        Ok(if other_is_greater { base } else { other })
    }

    fn max_method(base: Value, span: Span, mut inputs: Vec<Value>) -> syn::Result<Self> {
        if inputs.len() != 1 {
            return Err(Error::new(span, "expected one argument"));
        }

        let other = inputs.remove(0);

        let other_is_greater = Self::from_expr(
            &Expr::Bin(ExprBin {
                lhs: Box::new(Expr::Value(base.clone())),
                op: BinOp::Gt(Token![>](span)),
                rhs: Box::new(Expr::Value(other.clone())),
            }),
            &Namespace::new(),
        )?;

        let other_is_greater = match other_is_greater {
            Self::Bool(b) => b.value,
            _ => unreachable!(),
        };

        Ok(if other_is_greater { other } else { base })
    }

    fn clamp_method(base: Value, span: Span, mut inputs: Vec<Value>) -> syn::Result<Self> {
        if inputs.len() != 2 {
            return Err(Error::new(span, "expected 2 arguments"));
        }

        let max = inputs.remove(1);
        let min = inputs.remove(0);

        let min_is_greater = Self::from_expr(
            &Expr::Bin(ExprBin {
                lhs: Box::new(Expr::Value(base.clone())),
                op: BinOp::Gt(Token![>](span)),
                rhs: Box::new(Expr::Value(min.clone())),
            }),
            &Namespace::new(),
        )?;

        let max_is_greater = Self::from_expr(
            &Expr::Bin(ExprBin {
                lhs: Box::new(Expr::Value(base.clone())),
                op: BinOp::Gt(Token![>](span)),
                rhs: Box::new(Expr::Value(max.clone())),
            }),
            &Namespace::new(),
        )?;

        let min_is_greater = match min_is_greater {
            Self::Bool(b) => b.value,
            _ => unreachable!(),
        };

        let max_is_greater = match max_is_greater {
            Self::Bool(b) => b.value,
            _ => unreachable!(),
        };

        Ok(if min_is_greater {
            min
        } else if max_is_greater {
            base
        } else {
            max
        })
    }

    fn enumerate_method(base: Value, span: Span, inputs: Vec<Value>) -> syn::Result<Self> {
        if inputs.len() != 0 {
            return Err(Error::new(span, "expected zero arguments"));
        }

        Ok(match base {
            Self::List(base) => Self::List(ValueList {
                span: base.span,
                items: base
                    .items
                    .into_iter()
                    .enumerate()
                    .map(|(idx, item)| {
                        Value::List(ValueList {
                            span: item.span(),
                            items: vec![
                                Self::Int(LitInt::new(&idx.to_string(), item.span())),
                                item,
                            ],
                        })
                    })
                    .collect(),
            }),

            _ => return Err(Error::new_spanned(base, "expected a list")),
        })
    }

    fn index_method(base: Value, span: Span, inputs: Vec<Value>) -> syn::Result<Self> {
        if inputs.len() != 1 {
            return Err(Error::new(span, "expected one argument"));
        }

        let idx = match &inputs[0] {
            Value::Int(int) => int.base10_parse::<usize>().unwrap(),

            Value::List(list) => {
                return Ok(Self::List(ValueList {
                    span: list.span,
                    items: list
                        .clone()
                        .items
                        .into_iter()
                        .map(|idx| Self::index_method(base.clone(), span, vec![idx]))
                        .collect::<syn::Result<_>>()?,
                }));
            }

            input => return Err(Error::new_spanned(input, "expected an int")),
        };

        Ok(match base {
            Self::List(base) => match base.items.get(idx) {
                Some(item) => item.clone(),

                None => return Err(Error::new_spanned(&inputs[0], "out of bounds")),
            },

            _ => return Err(Error::new_spanned(base, "expected a list")),
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

    fn char_bin_op(lhs: char, op: BinOp, rhs: char) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn cstr_bin_op(lhs: &CStr, op: BinOp, rhs: &CStr) -> syn::Result<Self> {
        Ok(match op {
            BinOp::Eq(_) => bool(lhs == rhs, op.span()),
            BinOp::Ne(_) => bool(lhs != rhs, op.span()),
            BinOp::Lt(_) => bool(lhs < rhs, op.span()),
            BinOp::Gt(_) => bool(lhs > rhs, op.span()),
            BinOp::Le(_) => bool(lhs <= rhs, op.span()),
            BinOp::Ge(_) => bool(lhs >= rhs, op.span()),

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    fn byte_str_bin_op(lhs: &[u8], op: BinOp, rhs: &[u8]) -> syn::Result<Self> {
        Ok(match op {
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
