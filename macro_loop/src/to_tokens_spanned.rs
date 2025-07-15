use proc_macro2::{Span, TokenStream};

pub trait ToTokensSpanned: Sized {
    fn to_token_stream_spanned(&self, span: Span) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.to_tokens_spanned(span, &mut tokens);

        tokens
    }

    fn to_tokens_spanned(&self, span: Span, tokens: &mut TokenStream);
}
