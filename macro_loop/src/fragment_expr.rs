use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::token::Paren;

use super::{expr::*, fragment::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentExpr {
    #[paren]
    _parens: Paren,
    #[inside(_parens)]
    expr: Expr,
}

impl ApplyFragment for FragmentExpr {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let value = Value::from_expr(self.expr, names.clone())?;

        value.to_tokens(tokens);

        Ok(())
    }
}
