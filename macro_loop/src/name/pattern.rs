use derive_syn_parse::Parse;
use syn::{Error, Ident, Token, punctuated::Punctuated, token::Bracket};

use super::*;

#[derive(Clone, Parse)]
pub enum Pattern {
    #[peek(Ident, name = "identifer")]
    Ident(Name),

    #[allow(private_interfaces)]
    #[peek(Bracket, name = "list")]
    List(PatternList),
}

impl<'p, 'v> Namespace<'p, 'v> {
    pub fn insert_pat(&mut self, pat: &Pattern, value: ValueRef<'v>) -> syn::Result<()> {
        match pat {
            Pattern::Ident(self_) => self.insert(self_, value)?,

            Pattern::List(self_) => match value {
                ValueRef::Owned(Value::List(value)) => {
                    if self_.items.len() != value.items.len() {
                        return Err(Error::new_spanned(
                            value,
                            format!(
                                "value doesn't match pattern. expected {} items",
                                self_.items.len()
                            ),
                        ));
                    }

                    for (pat_item, value) in self_.items.iter().zip(value.items) {
                        self.insert_pat(pat_item, value)?;
                    }
                }

                ValueRef::Ref(Value::List(value)) => {
                    if self_.items.len() != value.items.len() {
                        return Err(Error::new_spanned(
                            value,
                            format!(
                                "value doesn't match pattern. expected {} items",
                                self_.items.len()
                            ),
                        ));
                    }

                    for (pat_item, value) in self_.items.iter().zip(&value.items) {
                        self.insert_pat(pat_item, ValueRef::Ref(value))?;
                    }
                }

                _ => {
                    return Err(Error::new_spanned(
                        &*value,
                        "value doesn't match pattern. expected a list",
                    ));
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Parse)]
struct PatternList {
    #[bracket]
    _brackets: Bracket,

    #[inside(_brackets)]
    #[call(Punctuated::parse_terminated)]
    items: Punctuated<Pattern, Token![,]>,
}
