use std::ffi::CStr;

use proc_macro2::Span;
use syn::{Error, LitBool, LitFloat, LitInt, LitStr};

use super::*;

impl<'a> Value<'a> {
    pub fn bin_op(&self, op: BinOp, rhs: &Self) -> syn::Result<Self> {
        Ok(match (self, rhs) {
            (Self::Bool(lhs), Self::Bool(rhs)) => Self::bool_bin_op(lhs.value, op, rhs.value)?,

            (Self::Int(lhs), Self::Int(rhs)) => {
                Self::int_bin_op(lhs.base10_parse::<u128>()?, op, rhs.base10_parse::<u128>()?)?
            }

            (Self::Float(lhs), Self::Float(rhs)) => {
                Self::float_bin_op(lhs.base10_parse::<f64>()?, op, rhs.base10_parse::<f64>()?)?
            }

            (Self::Str(lhs), Self::Str(rhs)) => Self::str_bin_op(&lhs.value(), op, &rhs.value())?,
            (Self::Str(lhs), Self::Ident(rhs)) => Self::str_bin_op(&lhs.value(), op, &rhs.str())?,
            (Self::Str(lhs), Self::Char(rhs)) => {
                Self::str_bin_op(&lhs.value(), op, &rhs.value().to_string())?
            }

            (Self::Char(lhs), Self::Char(rhs)) => Self::char_bin_op(lhs.value(), op, rhs.value())?,

            (Self::CStr(lhs), Self::CStr(rhs)) => {
                Self::cstr_bin_op(&lhs.value(), op, &rhs.value())?
            }

            (Self::ByteStr(lhs), Self::ByteStr(rhs)) => {
                Self::byte_str_bin_op(&lhs.value(), op, &rhs.value())?
            }

            (Self::Ident(lhs), Self::Ident(rhs)) => Self::ident_bin_op(&lhs.str(), op, &rhs.str())?,

            _ => return Err(Error::new_spanned(op, "invalid operation")),
        })
    }

    pub fn un_op(&self, op: UnOp) -> syn::Result<Self> {
        match self {
            _ => return Err(Error::new_spanned(op, "invalid operation")),
        }
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

            BinOp::Range(_) => Self::List(ValueList {
                span: op.span(),
                items: (lhs..rhs)
                    .map(|i| ValueRef::Owned(int(i, op.span())))
                    .collect(),
            }),

            BinOp::RangeInclusive(_) => Self::List(ValueList {
                span: op.span(),
                items: (lhs..=rhs)
                    .map(|i| ValueRef::Owned(int(i, op.span())))
                    .collect(),
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

fn int(value: u128, span: Span) -> Value<'static> {
    Value::Int(LitInt::new(&value.to_string(), span))
}

fn float(value: f64, span: Span) -> Value<'static> {
    Value::Float(LitFloat::new(&value.to_string(), span))
}

fn bool(value: bool, span: Span) -> Value<'static> {
    Value::Bool(LitBool { value, span })
}

fn string(value: impl AsRef<str>, span: Span) -> Value<'static> {
    Value::Str(LitStr::new(value.as_ref(), span))
}

fn ident(value: impl AsRef<str>, span: Span) -> Value<'static> {
    Value::Ident(IdentStr::new(value.as_ref(), span))
}
