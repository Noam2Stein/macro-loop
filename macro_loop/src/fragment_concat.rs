use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    Error, Ident, LitStr, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream, Parser},
    token::Bracket,
};

use super::{expr::*, fragment::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentConcat {
    #[bracket]
    _brackets: Bracket,
    #[inside(_brackets)]
    #[call(parse_segments)]
    segments: Vec<Segment>,
    #[inside(_brackets)]
    _type_arrow: Option<Token![=>]>,
    #[inside(_brackets)]
    #[parse_if(_type_arrow.is_some())]
    type_: Option<Ident>,
}

#[derive(Clone, Parse)]
enum Segment {
    #[peek_with(|input: ParseStream| input.peek(Ident::peek_any), name = "an ident")]
    Ident(#[call(Ident::parse_any)] Ident),

    #[peek(Token![@], name = "an ident")]
    Fragment(SegmentFragment),
}

#[derive(Clone, Parse)]
struct SegmentFragment {
    _at_token: Token![@],
    frag: Fragment,
}

impl ApplyFragment for FragmentConcat {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let str = self
            .segments
            .iter()
            .map(|seg| seg.try_to_string(names))
            .collect::<syn::Result<String>>()?;

        let span = self.segments.iter().map(|seg| seg.span()).nth(0).unwrap();

        let value = match self
            .type_
            .as_ref()
            .map(|type_| type_.to_string())
            .as_ref()
            .map(|str| str.as_str())
        {
            Some("str") => Value::Str(LitStr::new(&str, span)),
            None => Value::Ident(Ident::new(&str, span)),

            _ => return Err(Error::new_spanned(self.type_, "invalid concat type")),
        };

        value.to_tokens(tokens);

        Ok(())
    }
}

impl Segment {
    fn try_to_string(&self, names: &HashMap<String, Value>) -> syn::Result<String> {
        Ok(match self {
            Segment::Ident(ident) => ident.to_string(),
            Segment::Fragment(frag) => {
                let mut frag_output = TokenStream::new();
                frag.frag
                    .clone()
                    .apply(&mut names.clone(), &mut frag_output)?;

                let expr = Expr::parse.parse2(frag_output)?;

                let value = Value::from_expr(expr, names.clone())?;

                value.try_to_string()?
            }
        })
    }

    fn span(&self) -> Span {
        match self {
            Self::Ident(self_) => self_.span(),
            Segment::Fragment(self_) => self_._at_token.span,
        }
    }
}

fn parse_segments<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut items = Vec::new();

    items.push(input.parse()?);

    while !input.is_empty() && !input.peek(Token![=>]) {
        items.push(input.parse()?);
    }
    Ok(items)
}
