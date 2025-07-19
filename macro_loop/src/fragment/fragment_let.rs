use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::Token;

use super::*;

#[derive(Parse)]
pub struct FragLet {
    _let_token: Token![let],
    pat: Pattern,
    _eq_token: Token![=],
    value: Expr,
    _semi_token: Token![;],
}

impl ApplyFragment for FragLet {
    fn apply<'s: 'v, 'v>(
        &'s self,
        namespace: &mut Namespace<'v, 'v>,
        _tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = Value::from_expr(&self.value, &namespace)?;

        namespace.insert_pat(&self.pat, value)?;

        Ok(())
    }
}
