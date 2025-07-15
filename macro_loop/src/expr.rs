use std::collections::HashMap;

use derive_quote_to_tokens::ToTokens;
use derive_syn_parse::Parse;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, LitBool, LitByte, LitByteStr, LitCStr, LitChar, LitFloat, LitInt, LitStr, Token,
    parse::{Parse, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};

use super::{ops::*, value::*};

#[derive(Clone, ToTokens)]
pub enum Expr {
    Bool(LitBool),
    Int(LitInt),
    Float(LitFloat),
    Str(LitStr),
    Char(LitChar),
    CStr(LitCStr),
    ByteStr(LitByteStr),
    Ident(Ident),

    List(ExprList),
    Bin(ExprBin),
    Un(ExprUn),

    Name(ExprName),
}

#[derive(Clone)]
pub struct ExprList {
    pub span: Span,
    pub items: Vec<Expr>,
}

#[derive(Clone)]
pub struct ExprBin {
    pub lhs: Box<Expr>,
    pub op: BinOp,
    pub rhs: Box<Expr>,
}

#[derive(Clone)]
pub struct ExprUn {
    pub op: UnOp,
    pub base: Box<Expr>,
}

#[derive(Clone, Parse, ToTokens)]
pub struct ExprName {
    pub _at_token: Token![@],
    pub name: Ident,
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut output = Expr::parse_single(input)?;

        while let Some(op) = BinOp::option_parse(input) {
            let rhs = Expr::parse_single(input)?;

            if let Expr::Bin(ExprBin {
                lhs: _,
                op: output_op,
                rhs: ref mut output_rhs,
            }) = output
            {
                if output_op.lvl() > op.lvl() {
                    **output_rhs = Expr::Bin(ExprBin {
                        lhs: (*output_rhs).clone(),
                        op,
                        rhs: Box::new(rhs),
                    });
                } else {
                    output = Expr::Bin(ExprBin {
                        lhs: Box::new(output.clone()),
                        op,
                        rhs: Box::new(rhs),
                    });
                }
            } else {
                output = Expr::Bin(ExprBin {
                    lhs: Box::new(output.clone()),
                    op,
                    rhs: Box::new(rhs),
                });
            }
        }

        Ok(output)
    }
}

impl ToTokens for ExprList {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let group_stream = Punctuated::<_, Token![,]>::from_iter(&self.items).to_token_stream();

        let mut group = Group::new(proc_macro2::Delimiter::Bracket, group_stream);
        group.set_span(self.span.span());

        tokens.append(TokenTree::Group(group));
    }
}
impl ToTokens for ExprBin {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut group_stream = TokenStream::new();
        self.lhs.to_tokens(&mut group_stream);
        self.op.to_tokens(&mut group_stream);
        self.rhs.to_tokens(&mut group_stream);

        let group = Group::new(proc_macro2::Delimiter::Parenthesis, group_stream);

        tokens.append(TokenTree::Group(group));
    }
}
impl ToTokens for ExprUn {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut group_stream = TokenStream::new();
        self.op.to_tokens(&mut group_stream);
        self.base.to_tokens(&mut group_stream);

        let group = Group::new(proc_macro2::Delimiter::Parenthesis, group_stream);

        tokens.append(TokenTree::Group(group));
    }
}

impl ExprName {
    pub fn find(&self, names: &HashMap<String, Value>) -> syn::Result<Value> {
        match names.get(&self.name.to_string()) {
            Some(value) => Ok(value.clone()),
            None => Err(Error::new_spanned(
                &self.name,
                format!("can't find {}", self.name),
            )),
        }
    }
}

impl Expr {
    fn parse_single(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Some(op) = UnOp::option_parse(input) {
            let base = Box::new(Expr::parse_single(input)?);

            return Ok(Self::Un(ExprUn { op, base }));
        };

        if let Some(lit) = input.parse::<Option<LitBool>>()? {
            return Ok(Self::Bool(lit));
        };
        if let Some(lit) = input.parse::<Option<LitInt>>()? {
            return Ok(Self::Int(lit));
        };
        if let Some(lit) = input.parse::<Option<LitFloat>>()? {
            return Ok(Self::Float(lit));
        };
        if let Some(lit) = input.parse::<Option<LitStr>>()? {
            return Ok(Self::Str(lit));
        };
        if let Some(lit) = input.parse::<Option<LitChar>>()? {
            return Ok(Self::Char(lit));
        };
        if let Some(lit) = input.parse::<Option<LitCStr>>()? {
            return Ok(Self::CStr(lit));
        };
        if let Some(lit) = input.parse::<Option<LitByteStr>>()? {
            return Ok(Self::ByteStr(lit));
        };
        if let Some(ident) = input.parse::<Option<Ident>>()? {
            return Ok(Self::Ident(ident));
        };

        if let Some(lit) = input.parse::<Option<LitByte>>()? {
            return Ok(Self::Int(LitInt::new(&lit.value().to_string(), lit.span())));
        };

        if let Some(group) = input.parse::<Option<Group>>()? {
            return Ok(match group.delimiter() {
                Delimiter::None => {
                    return Err(Error::new(group.span(), "unsupported delimiters"));
                }

                Delimiter::Brace => {
                    return Err(Error::new(
                        group.delim_span().span(),
                        "blocks are currently unsupported",
                    ));
                }

                Delimiter::Bracket => {
                    let punctuated =
                        Punctuated::<_, Token![,]>::parse_terminated.parse2(group.stream())?;

                    Self::List(ExprList {
                        span: group.span(),
                        items: punctuated.into_iter().collect(),
                    })
                }

                Delimiter::Parenthesis => parse2(group.stream())?,
            });
        };

        if let Some(at_token) = input.parse::<Option<Token![@]>>()? {
            let name = input.parse()?;

            return Ok(Self::Name(ExprName {
                _at_token: at_token,
                name,
            }));
        };

        Err(input.error("expected an expression"))
    }
}
