use std::collections::HashMap;

use derive_quote_to_tokens::ToTokens;
use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Error, Ident, LitStr, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Bracket,
};

use super::{expr::*, fragment::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentConcat {
    #[bracket]
    _brackets: Bracket,
    #[inside(_brackets)]
    #[call(parse_segments)]
    segments: Vec<FragmentConcatSegment>,
    #[inside(_brackets)]
    _type_arrow: Option<Token![=>]>,
    #[inside(_brackets)]
    #[parse_if(_type_arrow.is_some())]
    type_: Option<Ident>,
}

#[derive(Clone, Parse, ToTokens)]
enum FragmentConcatSegment {
    #[peek_with(|input: ParseStream| input.peek(Ident::peek_any), name = "an ident")]
    Ident(#[call(Ident::parse_any)] Ident),

    #[peek(Token![@], name = "an ident")]
    Name(ExprName),
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

impl FragmentConcatSegment {
    fn try_to_string(&self, names: &HashMap<String, Value>) -> syn::Result<String> {
        Ok(match self {
            FragmentConcatSegment::Ident(ident) => ident.to_string(),
            FragmentConcatSegment::Name(name) => {
                let value = name.find(names)?;

                match value {
                    Value::Bool(lit) => lit.value.to_string(),
                    Value::Int(lit) => lit.base10_parse::<u128>().unwrap().to_string(),
                    Value::Str(lit) => lit.value(),
                    Value::Char(lit) => lit.value().to_string(),
                    Value::CStr(lit) => lit.value().to_str().unwrap().to_string(),
                    Value::ByteStr(lit) => String::from_utf8(lit.value()).unwrap(),
                    Value::Ident(ident) => ident.to_string(),

                    _ => {
                        return Err(Error::new_spanned(value, "not an identifier value"));
                    }
                }
            }
        })
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
