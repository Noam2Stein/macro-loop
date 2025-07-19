use derive_syn_parse::Parse;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    Error, Ident, LitStr, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream, Parser},
    token::Bracket,
};

use super::*;

#[derive(Parse)]
pub struct FragConcat {
    #[bracket]
    _brackets: Bracket,
    #[inside(_brackets)]
    #[call(parse_segments)]
    segments: Vec<Segment>,
    #[inside(_brackets)]
    _type_arrow: Option<Token![=>]>,
    #[inside(_brackets)]
    #[parse_if(_type_arrow.is_some())]
    type_: Option<IdentStr>,
}

#[derive(Parse)]
enum Segment {
    #[peek_with(|input: ParseStream| input.peek(Ident::peek_any), name = "an ident")]
    Ident(#[call(Ident::parse_any)] Ident),

    #[peek(Token![@], name = "an ident")]
    Fragment(SegmentFragment),
}

#[derive(Parse)]
struct SegmentFragment {
    _at_token: Token![@],
    frag: Frag,
}

impl ApplyFragment for FragConcat {
    fn apply<'s: 'v, 'v>(
        &'s self,
        namespace: &mut Namespace<'v, 'v>,
        tokens: &mut TokenStream,
    ) -> syn::Result<()> {
        let str = self
            .segments
            .iter()
            .map(|seg| seg.try_to_string(namespace))
            .collect::<syn::Result<String>>()?;

        let span = self.segments.iter().map(|seg| seg.span()).nth(0).unwrap();

        let value = match self.type_.as_ref().map(|type_| type_.str()) {
            Some("str") => Value::Str(LitStr::new(&str, span)),
            None => Value::Ident(IdentStr::new(str.into_boxed_str(), span)),

            _ => return Err(Error::new_spanned(&self.type_, "invalid concat type")),
        };

        value.to_tokens(tokens);

        Ok(())
    }
}

impl Segment {
    fn try_to_string(&self, namespace: &Namespace) -> syn::Result<String> {
        Ok(match self {
            Segment::Ident(ident) => ident.to_string(),
            Segment::Fragment(frag) => {
                let mut namespace = namespace.fork();

                let mut frag_output = TokenStream::new();
                frag.frag.apply(&mut namespace, &mut frag_output)?;

                let expr = Expr::parse.parse2(frag_output)?;

                let value = Value::from_expr(&expr, &namespace)?;

                value.try_to_string()?
            }
        })
    }

    fn span(&self) -> Span {
        match self {
            Self::Ident(self_) => self_.span(),
            Segment::Fragment(self_) => self_._at_token.span,
        }
    }
}

fn parse_segments<T: Parse>(input: ParseStream) -> syn::Result<Vec<T>> {
    let mut items = Vec::new();

    items.push(input.parse()?);

    while !input.is_empty() && !input.peek(Token![=>]) {
        items.push(input.parse()?);
    }
    Ok(items)
}
