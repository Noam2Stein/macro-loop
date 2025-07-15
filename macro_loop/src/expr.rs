use derive_quote_to_tokens::ToTokens;
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Error, Ident, Lit, Token,
    parse::{Parse, Parser},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
};

use super::op::*;

#[derive(Clone, ToTokens)]
pub enum Expr {
    Lit(Lit),
    Ident(Ident),
    List(ExprList),

    Name(ExprName),

    Bin(ExprBin),
    Un(ExprUn),
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

#[derive(Clone, ToTokens)]
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

impl Expr {
    fn parse_single(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Some(op) = UnOp::option_parse(input) {
            let base = Box::new(Expr::parse_single(input)?);

            return Ok(Self::Un(ExprUn { op, base }));
        };

        if let Some(lit) = input.parse::<Option<Lit>>()? {
            return Ok(Self::Lit(lit));
        };

        if let Some(ident) = input.parse::<Option<Ident>>()? {
            return Ok(Self::Ident(ident));
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
