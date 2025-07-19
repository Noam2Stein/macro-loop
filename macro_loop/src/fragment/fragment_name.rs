use derive_syn_parse::Parse;
use proc_macro2::TokenStream;

use super::*;

#[derive(Clone, Parse)]
pub struct FragName {
    name: Name,
}

impl ApplyFragment for FragName {
    fn apply<'s: 'v, 'v>(
        &'s self,
        namespace: &mut Namespace<'v, 'v>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = namespace.get(&self.name)?;

        value.to_tokens_spanned(self.name.span(), tokens);

        Ok(())
    }
}
