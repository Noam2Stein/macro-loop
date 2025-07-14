use std::{collections::HashMap, hash::Hash};

use derive_syn_parse::Parse;
use proc_macro2::Span;
use syn::{Error, Ident, Token, parse::Parse, punctuated::Punctuated, token::Bracket};

use super::value::*;

#[derive(Clone, Parse)]
pub enum Param {
    #[allow(private_interfaces)]
    #[peek(Ident, name = "identifer")]
    Ident(ParamIdent),

    #[allow(private_interfaces)]
    #[peek(Bracket, name = "list")]
    List(ParamList),
}

impl Param {
    pub fn insert_values(
        &self,
        value: Value,
        names: &mut HashMap<String, Value>,
    ) -> syn::Result<()> {
        for (key, value) in self.values(value)? {
            names.insert(key.str, value);
        }

        Ok(())
    }
}

#[derive(Clone)]
struct ParamIdent {
    str: String,
    span: Span,
}

#[derive(Clone, Parse)]
struct ParamList {
    #[bracket]
    _brackets: Bracket,

    #[inside(_brackets)]
    #[call(Punctuated::parse_terminated)]
    items: Punctuated<Param, Token![,]>,
}

impl Param {
    fn values(&self, value: Value) -> syn::Result<HashMap<ParamIdent, Value>> {
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
                    let item_names = item.values(value)?;

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

impl Parse for ParamIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        Ok(Self {
            str: ident.to_string(),
            span: ident.span(),
        })
    }
}

impl PartialEq for ParamIdent {
    fn eq(&self, other: &Self) -> bool {
        self.str == other.str
    }
}
impl Eq for ParamIdent {}

impl Hash for ParamIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.str.hash(state);
    }
}
