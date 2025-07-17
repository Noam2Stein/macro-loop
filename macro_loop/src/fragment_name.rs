use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::Ident;

use crate::to_tokens_spanned::ToTokensSpanned;

use super::{fragment::*, namespace::*};

#[derive(Clone, Parse)]
pub struct FragName {
    name: Ident,
}

impl ApplyFragment for FragName {
    fn apply(&self, namespace: &mut Namespace, tokens: &mut TokenStream) -> syn::Result<()> {
        let value = namespace.get(&self.name)?;

        value.to_tokens_spanned(self.name.span(), tokens);

        Ok(())
    }
}
