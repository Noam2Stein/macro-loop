use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::Token;

use super::{expr::*, fragment::*, namespace::*, pattern::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentLet {
    _let_token: Token![let],
    pat: Pattern,
    _eq_token: Token![=],
    value: Expr,
    _semi_token: Token![;],
}

impl ApplyFragment for FragmentLet {
    fn apply(self, namespace: &mut Namespace, _tokens: &mut TokenStream) -> syn::Result<()> {
        let value = Value::from_expr(self.value, &namespace)?;

        self.pat.insert_to_namespace(value, namespace)?;

        Ok(())
    }
}
