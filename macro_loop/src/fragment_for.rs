use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{Error, Token, punctuated::Punctuated, token::Brace};

use super::{expr::*, fragment::*, name_stream::*, namespace::*, pattern::*, value::*};

#[derive(Parse)]
pub struct FragFor {
    _for_token: Token![for],
    #[call(|input| Ok(Punctuated::<_, Token![,]>::parse_separated_nonempty(input)?.into_iter().collect()))]
    segments: Vec<FragForSegment>,
    #[brace]
    _braces: Brace,
    #[inside(_braces)]
    body: NameStream,
}

#[derive(Parse)]
struct FragForSegment {
    pat: Pattern,
    _in_token: Token![in],
    items: Expr,
}

impl ApplyFragment for FragFor {
    fn apply(&self, namespace: &mut Namespace, tokens: &mut TokenStream) -> syn::Result<()> {
        self.apply_inner(namespace, tokens, 0)
    }
}

impl FragFor {
    fn apply_inner(
        &self,
        namespace: &mut Namespace,
        tokens: &mut TokenStream,
        seg_idx: usize,
    ) -> syn::Result<()> {
        if self.segments.len() <= seg_idx {
            tokens.append_all(self.body.resolve(namespace)?);

            return Ok(());
        }

        let segment = &self.segments[seg_idx];

        let values = Value::from_expr(&segment.items, &namespace)?;

        let values = if let Value::List(values) = values {
            values.items
        } else {
            return Err(Error::new_spanned(&values, "expected a list"));
        };

        for value in values {
            let mut namespace = namespace.fork();

            segment.pat.insert_to_namespace(value, &mut namespace)?;

            self.apply_inner(&mut namespace, tokens, seg_idx + 1)?;
        }

        Ok(())
    }
}
