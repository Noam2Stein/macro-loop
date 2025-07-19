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

use super::*;

pub enum Expr {
    Value(Value<'static>),
    Frag(ExprFrag),
    Bin(Box<ExprBin>),
    Un(Box<ExprUn>),
    Method(ExprMethod),
    List(ExprList),
    Paren(Box<Expr>),
}

pub struct ExprList {
    pub span: Span,
    pub items: Vec<Expr>,
}

pub struct ExprBin {
    pub lhs: Expr,
    pub op: BinOp,
    pub rhs: Expr,
}

pub struct ExprUn {
    pub op: UnOp,
    pub base: Expr,
}

#[derive(Parse)]
pub struct ExprFrag {
    pub _at_token: Token![@],
    pub frag: Box<Frag>,
}

pub struct ExprMethod {
    pub base: Box<Expr>,
    pub method: IdentStr,
    pub inputs: Vec<Expr>,
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut output = Expr::parse_single(input)?;

        while let Some(op) = BinOp::option_parse(input) {
            let rhs = Expr::parse_single(input)?;

            if let Expr::Bin(ref mut bin) = output {
                let ExprBin {
                    lhs: _,
                    op: output_op,
                    rhs: ref mut output_rhs,
                } = **bin;

                if output_op.lvl() > op.lvl() {
                    output_rhs.try_replace(|output_rhs| Self::bin(output_rhs, op, rhs))?;
                } else {
                    output.try_replace(|output| Self::bin(output, op, rhs))?;
                }
            } else {
                output.try_replace(|output| Self::bin(output, op, rhs))?;
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

                let method = input.parse::<IdentStr>()?;

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
                        method: IdentStr::new("index", group.span()),
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
            let base = Expr::parse_single(input)?;

            return Self::un(op, base);
        };

        if let Some(lit) = input.parse::<Option<Lit>>()? {
            return Ok(Self::Value(Value::from_lit(lit)?));
        };

        if input.peek(Ident) {
            return Ok(Self::Value(Value::Ident(input.parse().unwrap())));
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

    fn bin(self, op: BinOp, rhs: Self) -> syn::Result<Self> {
        Ok(
            if let (Self::Value(self_), Self::Value(rhs)) = (&self, &rhs) {
                Self::Value(self_.bin_op(op, rhs)?)
            } else {
                Self::Bin(Box::new(ExprBin { lhs: self, op, rhs }))
            },
        )
    }

    fn un(op: UnOp, base: Self) -> syn::Result<Self> {
        Ok(if let Self::Value(base) = base {
            Self::Value(base.un_op(op)?)
        } else {
            Self::Un(Box::new(ExprUn { op, base }))
        })
    }

    fn replace(&mut self, value: impl FnOnce(Self) -> Self) {
        *self = value(replace(self, Self::temporary()));
    }

    fn try_replace(&mut self, value: impl FnOnce(Self) -> syn::Result<Self>) -> syn::Result<()> {
        *self = value(replace(self, Self::temporary()))?;

        Ok(())
    }

    fn temporary() -> Self {
        Self::Value(Value::Bool(LitBool::new(false, Span::call_site())))
    }
}
