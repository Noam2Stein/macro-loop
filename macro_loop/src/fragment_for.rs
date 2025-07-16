use std::collections::HashMap;

use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{
    Error, Token,
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    token::Brace,
};

use super::{expr::*, fragment::*, map::*, pattern::*, value::*};

#[derive(Clone, Parse)]
pub struct FragmentFor {
    _for_token: Token![for],
    #[call(|input| Ok(Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?.into_iter().collect()))]
    segments: Vec<FragmentForSegment>,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: TokenStream,
}

#[derive(Clone, Parse)]
struct FragmentForSegment {
    pat: Pattern,
    _in_token: Token![in],
    items: Expr,
}

impl ApplyFragment for FragmentFor {
    fn apply(
        mut self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        if self.segments.len() == 0 {
            let map_fn = |input: ParseStream| map_tokenstream(input, names.clone());
            tokens.append_all(map_fn.parse2(self.body.clone())?);

            return Ok(());
        }

        let segment = self.segments.remove(0);

        let values = Value::from_expr(segment.items.clone(), names.clone())?;

        let values = if let Value::List(values) = values {
            values.items
        } else {
            return Err(Error::new_spanned(&values, "expected a list"));
        };

        for value in values {
            let mut names = names.clone();

            segment.pat.insert_to_names(value, &mut names)?;

            self.clone().apply(&mut names, tokens)?;
        }

        Ok(())
    }
}
