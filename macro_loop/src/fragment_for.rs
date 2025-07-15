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
    #[call(Punctuated::parse_separated_nonempty)]
    segments: Punctuated<FragmentForSegment, Token![,]>,
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
        self,
        names: &mut HashMap<String, Value>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let lists = self
            .segments
            .iter()
            .map(|segment| Value::from_expr(segment.items.clone(), names.clone()))
            .collect::<syn::Result<Vec<Value>>>()?;

        let lists = lists
            .into_iter()
            .map(|items| {
                if let Value::List(items) = items {
                    Ok(items.items)
                } else {
                    Err(Error::new_spanned(&items, "expected a list"))
                }
            })
            .collect::<syn::Result<Vec<Vec<Value>>>>()?;

        for values in cartesian_product(lists) {
            let mut names = names.clone();

            for (pat, value) in self.segments.iter().map(|seg| &seg.pat).zip(values) {
                pat.insert_to_names(value, &mut names)?;
            }

            let map_fn = |input: ParseStream| map_tokenstream(input, names);
            tokens.append_all(map_fn.parse2(self.body.clone())?);
        }

        Ok(())
    }
}

fn cartesian_product<T: Clone>(input: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![vec![]];

    for pool in input {
        let mut new_result = Vec::new();
        for combination in &result {
            for item in &pool {
                let mut new_combination = combination.clone();
                new_combination.push(item.clone());
                new_result.push(new_combination);
            }
        }
        result = new_result;
    }

    result
}
