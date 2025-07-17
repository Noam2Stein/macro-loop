use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::token::Paren;

use super::{expr::*, fragment::*, namespace::*, value::*};

#[derive(Parse)]
pub struct FragExpr {
    #[paren]
    _parens: Paren,
    #[inside(_parens)]
    expr: Expr,
}

impl ApplyFragment for FragExpr {
    fn apply(&self, namespace: &mut Namespace, tokens: &mut TokenStream) -> syn::Result<()> {
        let value = Value::from_expr(&self.expr, &namespace)?;

        value.to_tokens(tokens);

        Ok(())
    }
}
