use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::token::Paren;

use super::{expr::*, fragment::*, namespace::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentExpr {
    #[paren]
    _parens: Paren,
    #[inside(_parens)]
    expr: Expr,
}

impl ApplyFragment for FragmentExpr {
    fn apply(self, namespace: &mut Namespace, tokens: &mut TokenStream) -> syn::Result<()> {
        let value = Value::from_expr(self.expr, &namespace)?;

        value.to_tokens(tokens);

        Ok(())
    }
}
