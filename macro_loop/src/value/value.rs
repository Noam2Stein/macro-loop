use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    Error, Lit, LitBool, LitByteStr, LitCStr, LitChar, LitFloat, LitInt, LitStr,
    parse::{Parse, Parser},
};

use super::*;

#[derive(Clone)]
pub enum Value<'v> {
    Bool(LitBool),
    Int(LitInt),
    Float(LitFloat),
    Str(LitStr),
    Char(LitChar),
    CStr(LitCStr),
    ByteStr(LitByteStr),
    Ident(IdentStr),

    List(ValueList<'v>),
}

impl<'a> ToTokens for Value<'a> {
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

impl<'a> ToTokensSpanned for Value<'a> {
    fn to_tokens_spanned(&self, span: Span, tokens: &mut TokenStream) {
        macro_rules! token_output {
            ($self_:ident) => {{
                let mut self_ = $self_.clone();
                self_.set_span(span);
                self_.to_tokens(tokens);
            }};
        }

        match self {
            Self::List(list) => list.to_tokens_spanned(span, tokens),

            Self::Bool(self_) => token_output!(self_),
            Self::Int(self_) => token_output!(self_),
            Self::Float(self_) => token_output!(self_),
            Self::Str(self_) => token_output!(self_),
            Self::Char(self_) => token_output!(self_),
            Self::CStr(self_) => token_output!(self_),
            Self::ByteStr(self_) => token_output!(self_),
            Self::Ident(self_) => token_output!(self_),
        }
    }
}

impl<'v> Value<'v> {
    pub fn from_expr(expr: &'v Expr, namespace: &Namespace<'v, 'v>) -> syn::Result<ValueRef<'v>> {
        Ok(match expr {
            Expr::Paren(inner) => Self::from_expr(inner, namespace)?,

            Expr::Value(value) => ValueRef::Ref(value),

            Expr::List(list) => ValueRef::Owned(Self::List(ValueList {
                span: list.span,
                items: list
                    .items
                    .iter()
                    .map(|item| Self::from_expr(item, namespace))
                    .collect::<syn::Result<_>>()?,
            })),

            Expr::Bin(bin) => {
                let ExprBin { lhs, op, rhs } = &**bin;

                let lhs = Value::from_expr(lhs, namespace)?;
                let rhs = Value::from_expr(rhs, namespace)?;

                ValueRef::Owned(lhs.bin_op(*op, &rhs)?)
            }

            Expr::Un(un) => {
                let ExprUn { op, base } = &**un;

                let base = Value::from_expr(base, namespace)?;

                ValueRef::Owned(base.un_op(*op)?)
            }

            Expr::Method(expr) => {
                let base = Value::from_expr(&expr.base, namespace)?;
                let inputs = expr
                    .inputs
                    .iter()
                    .map(|input| Value::from_expr(input, namespace))
                    .collect::<syn::Result<Vec<_>>>()?;

                base.method(&expr.method, &inputs)?
            }

            Expr::Frag(ExprFrag { _at_token: _, frag }) => {
                let expr = {
                    let mut namespace = namespace.fork();

                    let mut output = TokenStream::new();
                    frag.apply(&mut namespace, &mut output)?;

                    Expr::parse.parse2(output)?
                };

                Value::from_owned_expr(expr, &namespace)?
            }
        })
    }

    pub fn from_owned_expr(expr: Expr, namespace: &Namespace<'v, 'v>) -> syn::Result<ValueRef<'v>> {
        Ok(match expr {
            Expr::Paren(inner) => Self::from_owned_expr(*inner, namespace)?,

            Expr::Value(value) => ValueRef::Owned(value),

            Expr::List(list) => ValueRef::Owned(Self::List(ValueList {
                span: list.span,
                items: list
                    .items
                    .into_iter()
                    .map(|item| Self::from_owned_expr(item, namespace))
                    .collect::<syn::Result<_>>()?,
            })),

            Expr::Bin(bin) => {
                let ExprBin { lhs, op, rhs } = *bin;

                let lhs = Value::from_owned_expr(lhs, namespace)?;
                let rhs = Value::from_owned_expr(rhs, namespace)?;

                ValueRef::Owned(lhs.bin_op(op, &rhs)?)
            }

            Expr::Un(un) => {
                let ExprUn { op, base } = *un;

                let base = Value::from_owned_expr(base, namespace)?;

                ValueRef::Owned(base.un_op(op)?)
            }

            Expr::Method(expr) => {
                let base = Value::from_owned_expr(*expr.base, namespace)?;

                let inputs = expr
                    .inputs
                    .into_iter()
                    .map(|input| Value::from_owned_expr(input, namespace))
                    .collect::<syn::Result<Vec<_>>>()?;

                base.method(&expr.method, &inputs)?
            }

            Expr::Frag(ExprFrag { _at_token: _, frag }) => {
                let expr = {
                    let mut namespace = namespace.fork();

                    let mut output = TokenStream::new();
                    frag.apply(&mut namespace, &mut output)?;

                    Expr::parse.parse2(output)?
                };

                Value::from_owned_expr(expr, &namespace)?
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
            Self::Ident(ident) => ident.str().to_string(),

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
}
