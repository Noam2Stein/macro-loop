use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{Error, Token, token::Brace};

use super::*;

#[derive(Parse)]
pub struct FragIf {
    _if_token: Token![if],
    condition: Expr,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: NameStream,
}

impl ApplyFragment for FragIf {
    fn apply<'s: 'v, 'v>(
        &'s self,
        namespace: &mut Namespace<'v, 'v>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let condition = Value::from_expr(&self.condition, &namespace)?;

        let condition = match &*condition {
            Value::Bool(condition) => condition.value,
            _ => return Err(Error::new_spanned(&self._if_token, "expected a bool")),
        };

        if condition {
            tokens.append_all(self.body.resolve(namespace)?);
        }

        Ok(())
    }
}
