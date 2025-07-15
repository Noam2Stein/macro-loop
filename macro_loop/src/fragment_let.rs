use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::Token;

use super::{expr::*, fragment::*, pattern::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentLet {
    _let_token: Token![let],
    pat: Pattern,
    _eq_token: Token![=],
    value: Expr,
    _semi_token: Token![;],
}

impl ApplyFragment for FragmentLet {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        _tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = Value::from_expr(self.value, names.clone())?;

        self.pat.insert_to_names(value, names)?;

        Ok(())
    }
}
