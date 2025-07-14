use proc_macro2::{Group, TokenTree, extra::DelimSpan};
use quote::{ToTokens, TokenStreamExt};
use syn::{Error, Lit, Token, parse::Parse, punctuated::Punctuated, spanned::Spanned};

use super::{expr::*, op::*};

#[derive(Clone)]
pub enum Value {
    Lit(Lit),
    List(ValueList),
}

#[derive(Clone)]
pub struct ValueList {
    pub span: DelimSpan,
    pub items: Vec<Value>,
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Lit(lit) => lit.to_tokens(tokens),

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

impl Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self::from_expr(Expr::parse(input)?)?)
    }
}

impl Value {
    fn from_expr(expr: Expr) -> syn::Result<Self> {
        Ok(match expr {
            Expr::Value(value) => value,

            Expr::BinOp(ExprBin { lhs, op, rhs }) => {
                let lhs = Value::from_expr(*lhs)?;
                let rhs = Value::from_expr(*rhs)?;

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

            Expr::UnOp(ExprUn { op, base }) => {
                let base = Value::from_expr(*base)?;

                match op {
                    UnOp::Neg(op) => Self::neg(op, base)?,
                    UnOp::Not(op) => Self::not(op, base)?,
                }
            }
        })
    }

    fn add(lhs: Self, op: Token![+], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't add lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't add bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't add bytes")),
            Value::Lit(Lit::ByteStr(_)) => return Err(Error::new_spanned(op, "can't add strings")),
            Value::Lit(Lit::CStr(_)) => return Err(Error::new_spanned(op, "can't add strings")),
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't add chars")),
            Value::Lit(Lit::Float(_)) => return Err(Error::new_spanned(op, "can't add floats")),
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't add ints")),
            Value::Lit(Lit::Str(_)) => return Err(Error::new_spanned(op, "can't add strings")),
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn sub(lhs: Self, op: Token![-], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't subtract lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't subtract bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't subtract bytes")),
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't subtract strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't subtract strings"));
            }
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't subtract chars")),
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't subtract floats"));
            }
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't subtract ints")),
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't subtract strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn mul(lhs: Self, op: Token![*], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't multiply lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't multiply bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't multiply bytes")),
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't multiply strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't multiply strings"));
            }
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't multiply chars")),
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't multiply floats"));
            }
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't multiply ints")),
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't multiply strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn div(lhs: Self, op: Token![/], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't divide lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't divide bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't divide bytes")),
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't divide strings"));
            }
            Value::Lit(Lit::CStr(_)) => return Err(Error::new_spanned(op, "can't divide strings")),
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't divide chars")),
            Value::Lit(Lit::Float(_)) => return Err(Error::new_spanned(op, "can't divide floats")),
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't divide ints")),
            Value::Lit(Lit::Str(_)) => return Err(Error::new_spanned(op, "can't divide strings")),
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn rem(lhs: Self, op: Token![%], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't remainder lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't remainder bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't remainder bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't remainder strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't remainder strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't remainder chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't remainder floats"));
            }
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't remainder ints")),
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't remainder strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn and(lhs: Self, op: Token![&], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't bitwise-and lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and floats"));
            }
            Value::Lit(Lit::Int(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and ints"));
            }
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-and strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn or(lhs: Self, op: Token![|], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't bitwise-or lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or floats"));
            }
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't bitwise-or ints")),
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-or strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn xor(lhs: Self, op: Token![^], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't bitwise-xor lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor floats"));
            }
            Value::Lit(Lit::Int(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor ints"));
            }
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't bitwise-xor strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn shl(lhs: Self, op: Token![<<], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't shift-left lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left floats"));
            }
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't shift-left ints")),
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't shift-left strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn shr(lhs: Self, op: Token![>>], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't shift-right lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right floats"));
            }
            Value::Lit(Lit::Int(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right ints"));
            }
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't shift-right strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn range(lhs: Self, op: Token![..], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't range lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't range bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't range bytes")),
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't range strings"));
            }
            Value::Lit(Lit::CStr(_)) => return Err(Error::new_spanned(op, "can't range strings")),
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't range chars")),
            Value::Lit(Lit::Float(_)) => return Err(Error::new_spanned(op, "can't range floats")),
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't range ints")),
            Value::Lit(Lit::Str(_)) => return Err(Error::new_spanned(op, "can't range strings")),
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn range_inclusive(lhs: Self, op: Token![..=], rhs: Self) -> syn::Result<Self> {
        Ok(match lhs {
            Value::List(_) => return Err(Error::new_spanned(op, "can't range lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't range bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't range bytes")),
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't range strings"));
            }
            Value::Lit(Lit::CStr(_)) => return Err(Error::new_spanned(op, "can't range strings")),
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't range chars")),
            Value::Lit(Lit::Float(_)) => return Err(Error::new_spanned(op, "can't range floats")),
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't range ints")),
            Value::Lit(Lit::Str(_)) => return Err(Error::new_spanned(op, "can't range strings")),
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn neg(op: Token![-], base: Self) -> syn::Result<Self> {
        Ok(match base {
            Value::List(_) => return Err(Error::new_spanned(op, "can't neg lists")),
            Value::Lit(Lit::Bool(_)) => return Err(Error::new_spanned(op, "can't neg bools")),
            Value::Lit(Lit::Byte(_)) => return Err(Error::new_spanned(op, "can't neg bytes")),
            Value::Lit(Lit::ByteStr(_)) => return Err(Error::new_spanned(op, "can't neg strings")),
            Value::Lit(Lit::CStr(_)) => return Err(Error::new_spanned(op, "can't neg strings")),
            Value::Lit(Lit::Char(_)) => return Err(Error::new_spanned(op, "can't neg chars")),
            Value::Lit(Lit::Float(_)) => return Err(Error::new_spanned(op, "can't neg floats")),
            Value::Lit(Lit::Int(_)) => return Err(Error::new_spanned(op, "can't neg ints")),
            Value::Lit(Lit::Str(_)) => return Err(Error::new_spanned(op, "can't neg strings")),
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }

    fn not(op: Token![!], base: Self) -> syn::Result<Self> {
        Ok(match base {
            Value::List(_) => return Err(Error::new_spanned(op, "can't apply not on lists")),
            Value::Lit(Lit::Bool(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on bools"));
            }
            Value::Lit(Lit::Byte(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on bytes"));
            }
            Value::Lit(Lit::ByteStr(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on strings"));
            }
            Value::Lit(Lit::CStr(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on strings"));
            }
            Value::Lit(Lit::Char(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on chars"));
            }
            Value::Lit(Lit::Float(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on floats"));
            }
            Value::Lit(Lit::Int(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on ints"));
            }
            Value::Lit(Lit::Str(_)) => {
                return Err(Error::new_spanned(op, "can't apply not on strings"));
            }
            Value::Lit(_) => return Err(Error::new_spanned(op, "unsupported literal")),
        })
    }
}
