use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::{Error, Ident};

use crate::to_tokens_spanned::ToTokensSpanned;

use super::{fragment::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentName {
    name: Ident,
}

impl ApplyFragment for FragmentName {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = match names.get(&self.name.to_string()) {
            Some(value) => value,
            None => {
                return Err(Error::new_spanned(
                    &self.name,
                    format!("can't find {}", self.name),
                ));
            }
        };

        value.to_tokens_spanned(self.name.span(), tokens);

        Ok(())
    }
}
