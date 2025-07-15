use std::{collections::HashMap, hash::Hash};

use derive_syn_parse::Parse;
use proc_macro2::Span;
use syn::{Error, Ident, Token, parse::Parse, punctuated::Punctuated, token::Bracket};

use super::value::*;

#[derive(Clone, Parse)]
pub enum Pattern {
    #[allow(private_interfaces)]
    #[peek(Ident, name = "identifer")]
    Ident(PatternIdent),

    #[allow(private_interfaces)]
    #[peek(Bracket, name = "list")]
    List(PatternList),
}

impl Pattern {
    pub fn insert_to_names(
        &self,
        value: Value,
        names: &mut HashMap<String, Value>,
    ) -> syn::Result<()> {
        for (key, value) in self.names(value)? {
            names.insert(key.str, value);
        }

        Ok(())
    }
}

#[derive(Clone)]
struct PatternIdent {
    str: String,
    span: Span,
}

#[derive(Clone, Parse)]
struct PatternList {
    #[bracket]
    _brackets: Bracket,

    #[inside(_brackets)]
    #[call(Punctuated::parse_terminated)]
    items: Punctuated<Pattern, Token![,]>,
}

impl Pattern {
    fn names(&self, value: Value) -> syn::Result<HashMap<PatternIdent, Value>> {
        Ok(match self {
            Self::Ident(ident) => HashMap::from_iter([(ident.clone(), value)]),

            Self::List(list) => {
                let value = if let Value::List(value) = value {
                    value
                } else {
                    return Err(Error::new_spanned(value, "expected a list"));
                };

                let mut output = HashMap::new();

                if list.items.len() != value.items.len() {
                    return Err(Error::new_spanned(
                        value,
                        format!("expected {} items", list.items.len()),
                    ));
                }

                for (item, value) in list.items.iter().zip(value.items) {
                    let item_names = item.names(value)?;

                    for (key, value) in &item_names {
                        if output.contains_key(key) {
                            return Err(Error::new(key.span, "duplicate names"));
                        }

                        output.insert(key.clone(), value.clone());
                    }
                }

                output
            }
        })
    }
}

impl Parse for PatternIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        Ok(Self {
            str: ident.to_string(),
            span: ident.span(),
        })
    }
}

impl PartialEq for PatternIdent {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}
impl Eq for PatternIdent {}

impl Hash for PatternIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.str.hash(state);
    }
}
