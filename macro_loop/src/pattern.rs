use derive_syn_parse::Parse;
use syn::{Error, Ident, Token, punctuated::Punctuated, token::Bracket};

use super::{namespace::*, value::*};

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
    pub fn insert_to_namespace(&self, value: Value, namespace: &mut Namespace) -> syn::Result<()> {
        match self {
            Self::Ident(self_) => namespace.insert(self_.ident.clone(), value)?,

            Self::List(self_) => {
                let value = if let Value::List(value) = value {
                    value
                } else {
                    return Err(Error::new_spanned(
                        value,
                        "value doesn't match pattern. expected a list",
                    ));
                };

                if self_.items.len() != value.items.len() {
                    return Err(Error::new_spanned(
                        value,
                        format!(
                            "value doesn't match pattern. expected {} items",
                            self_.items.len()
                        ),
                    ));
                }

                for (item, value) in self_.items.iter().zip(value.items) {
                    item.insert_to_namespace(value, namespace)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Parse)]
struct PatternIdent {
    ident: Ident,
}

#[derive(Clone, Parse)]
struct PatternList {
    #[bracket]
    _brackets: Bracket,

    #[inside(_brackets)]
    #[call(Punctuated::parse_terminated)]
    items: Punctuated<Pattern, Token![,]>,
}
