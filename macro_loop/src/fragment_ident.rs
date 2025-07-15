use std::collections::HashMap;

use derive_quote_to_tokens::ToTokens;
use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Error, Ident, Lit, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Bracket,
};

use super::{expr::*, fragment::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentIdent {
    #[bracket]
    _brackets: Bracket,
    #[inside(_brackets)]
    #[call(parse_repeated_nonempty)]
    segments: Vec<FragmentIdentSegment>,
}

#[derive(Clone, Parse, ToTokens)]
enum FragmentIdentSegment {
    #[peek(Ident, name = "an ident")]
    Ident(Ident),

    #[peek(Token![@], name = "an ident")]
    Name(ExprName),
}

impl ApplyFragment for FragmentIdent {
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

        tokens.append(Ident::new(&str, span));

        Ok(())
    }
}

impl FragmentIdentSegment {
    fn try_to_string(&self, names: &HashMap<String, Value>) -> syn::Result<String> {
        Ok(match self {
            FragmentIdentSegment::Ident(ident) => ident.to_string(),
            FragmentIdentSegment::Name(name) => {
                let value = name.find(names)?;

                match value {
                    Value::Ident(ident) => ident.to_string(),

                    Value::Lit(Lit::Bool(lit)) => lit.value.to_string(),
                    Value::Lit(Lit::Byte(lit)) => (lit.value() as char).to_string(),
                    Value::Lit(Lit::ByteStr(lit)) => String::from_utf8(lit.value()).unwrap(),
                    Value::Lit(Lit::CStr(lit)) => lit.value().to_str().unwrap().to_string(),
                    Value::Lit(Lit::Char(lit)) => lit.value().to_string(),
                    Value::Lit(Lit::Int(lit)) => lit.base10_parse::<u128>().unwrap().to_string(),
                    Value::Lit(Lit::Str(lit)) => lit.value(),

                    Value::List(_) | Value::Lit(_) => {
                        return Err(Error::new_spanned(value, "not an identifier value"));
                    }
                }
            }
        })
    }
}

fn parse_repeated_nonempty<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut items = Vec::new();

    items.push(input.parse()?);

    while !input.is_empty() {
        items.push(input.parse()?);
    }
    Ok(items)
}
