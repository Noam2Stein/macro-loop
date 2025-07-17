use std::mem::replace;

use derive_syn_parse::Parse;
use proc_macro2::{Delimiter, Group, Span};
use syn::{
    Error, Ident, Lit, LitBool, Token,
    parse::{Parse, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Bracket,
};

use super::{fragment::*, ops::*, value::*};

pub enum Expr {
    Value(Value),
    Frag(ExprFrag),
    Bin(ExprBin),
    Un(ExprUn),
    Method(ExprMethod),
    List(ExprList),
    Paren(Box<Expr>),
}

pub struct ExprList {
    pub span: Span,
    pub items: Vec<Expr>,
}

pub struct ExprBin {
    pub lhs: Box<Expr>,
    pub op: BinOp,
    pub rhs: Box<Expr>,
}

pub struct ExprUn {
    pub op: UnOp,
    pub base: Box<Expr>,
}

#[derive(Parse)]
pub struct ExprFrag {
    pub _at_token: Token![@],
    pub frag: Box<Frag>,
}

pub struct ExprMethod {
    pub base: Box<Expr>,
    pub method: Ident,
    pub inputs: Vec<Expr>,
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
                    output_rhs.replace(|output_rhs| {
                        Expr::Bin(ExprBin {
                            lhs: Box::new(output_rhs),
                            op,
                            rhs: Box::new(rhs),
                        })
                    });
                } else {
                    output.replace(|output| {
                        Expr::Bin(ExprBin {
                            lhs: Box::new(output),
                            op,
                            rhs: Box::new(rhs),
                        })
                    });
                }
            } else {
                output.replace(|output| {
                    Expr::Bin(ExprBin {
                        lhs: Box::new(output),
                        op,
                        rhs: Box::new(rhs),
                    })
                });
            }
        }

        Ok(output)
    }
}

impl Expr {
    fn parse_single(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut output = Self::parse_base(input)?;

        loop {
            if input.peek(Token![.]) && !input.peek(Token![..]) {
                input.parse::<Token![.]>().unwrap();

                let method = input.parse::<Ident>()?;

                let inputs = input.parse::<Group>()?;
                if inputs.delimiter() != Delimiter::Parenthesis {
                    return Err(Error::new(inputs.span(), "expected `()`"));
                }

                let inputs = Punctuated::<Expr, Token![,]>::parse_terminated
                    .parse2(inputs.stream())?
                    .into_iter()
                    .collect();

                output.replace(|output| {
                    Self::Method(ExprMethod {
                        base: Box::new(output),
                        method,
                        inputs,
                    })
                });

                continue;
            }

            if input.peek(Bracket) {
                let group = input.parse::<Group>()?;

                let idx = Expr::parse.parse2(group.stream())?;

                output.replace(|output| {
                    Self::Method(ExprMethod {
                        base: Box::new(output),
                        method: Ident::new("index", group.span()),
                        inputs: vec![idx],
                    })
                });
            }

            break;
        }

        Ok(output)
    }

    fn parse_base(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Some(op) = UnOp::option_parse(input) {
            let base = Box::new(Expr::parse_single(input)?);

            return Ok(Self::Un(ExprUn { op, base }));
        };

        if let Some(lit) = input.parse::<Option<Lit>>()? {
            return Ok(Self::Value(Value::from_lit(lit)?));
        };

        if let Some(ident) = input.parse::<Option<Ident>>()? {
            return Ok(Self::Value(Value::Ident(ident)));
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

                Delimiter::Parenthesis => Self::Paren(Box::new(parse2(group.stream())?)),
            });
        };

        if let Some(at_token) = input.parse::<Option<Token![@]>>()? {
            let frag = input.parse()?;

            return Ok(Self::Frag(ExprFrag {
                _at_token: at_token,
                frag,
            }));
        };

        Err(input.error("expected an expression"))
    }

    fn temporary() -> Self {
        Self::Value(Value::Bool(LitBool::new(false, Span::call_site())))
    }

    fn replace(&mut self, value: impl FnOnce(Self) -> Self) {
        *self = value(replace(self, Self::temporary()));
    }
}
