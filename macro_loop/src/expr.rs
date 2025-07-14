use proc_macro2::{Delimiter, Group};
use syn::{
    Error, Lit, Token,
    parse::{Parse, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};

use super::{op::*, value::*};

#[derive(Clone)]
pub enum Expr {
    Value(Value),
    BinOp(ExprBin),
    UnOp(ExprUn),
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

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut output = Expr::parse_single(input)?;

        while let Some(op) = BinOp::option_parse(input) {
            let rhs = Expr::parse_single(input)?;

            if let Expr::BinOp(ExprBin {
                lhs: _,
                op: output_op,
                rhs: ref mut output_rhs,
            }) = output
            {
                if output_op.lvl() > op.lvl() {
                    **output_rhs = Expr::BinOp(ExprBin {
                        lhs: (*output_rhs).clone(),
                        op,
                        rhs: Box::new(rhs),
                    });
                } else {
                    output = Expr::BinOp(ExprBin {
                        lhs: Box::new(output.clone()),
                        op,
                        rhs: Box::new(rhs),
                    });
                }
            } else {
                output = Expr::BinOp(ExprBin {
                    lhs: Box::new(output.clone()),
                    op,
                    rhs: Box::new(rhs),
                });
            }
        }

        Ok(output)
    }
}

impl Expr {
    fn parse_single(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Some(op) = UnOp::option_parse(input) {
            let base = Box::new(Expr::parse_single(input)?);

            return Ok(Self::UnOp(ExprUn { op, base }));
        };

        if let Some(lit) = input.parse::<Option<Lit>>()? {
            return Ok(Expr::Value(Value::Lit(lit)));
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
                        Punctuated::<Value, Token![,]>::parse_terminated.parse2(group.stream())?;

                    Self::Value(Value::List(ValueList {
                        span: group.delim_span(),
                        items: punctuated.into_iter().collect(),
                    }))
                }

                Delimiter::Parenthesis => {
                    let value = parse2::<Value>(group.stream())?;

                    Self::Value(value)
                }
            });
        };

        Err(input.error("expected an expression"))
    }
}
