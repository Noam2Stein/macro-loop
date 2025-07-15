use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Error, Lit, Token,
    parse::{ParseStream, Parser},
    token::Brace,
};

use super::{expr::*, fragment::*, map::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentIf {
    _if_token: Token![if],
    condition: Expr,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: TokenStream,
}

impl ApplyFragment for FragmentIf {
    fn apply(
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let condition = Value::from_expr(self.condition, names.clone())?;

        let condition = match condition {
            Value::Lit(Lit::Bool(condition)) => condition.value,
            condition => return Err(Error::new_spanned(condition, "expected a bool")),
        };

        if condition {
            let map_fn = |input: ParseStream| map_tokenstream(input, names.clone());
            tokens.append_all(map_fn.parse2(self.body.clone())?);
        }

        Ok(())
    }
}
