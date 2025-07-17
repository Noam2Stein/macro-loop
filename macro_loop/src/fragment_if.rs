use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Error, Token,
    parse::{ParseStream, Parser},
    token::Brace,
};

use super::{expr::*, fragment::*, map::*, namespace::*, value::*};

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
    fn apply(self, namespace: &mut Namespace, tokens: &mut TokenStream) -> syn::Result<()> {
        let condition = Value::from_expr(self.condition.clone(), &namespace)?;

        let condition = match condition {
            Value::Bool(condition) => condition.value,
            _ => return Err(Error::new_spanned(self.condition, "expected a bool")),
        };

        if condition {
            let map_fn = |input: ParseStream| map_tokenstream(input, &namespace);
            tokens.append_all(map_fn.parse2(self.body.clone())?);
        }

        Ok(())
    }
}
