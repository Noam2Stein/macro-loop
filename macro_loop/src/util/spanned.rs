use proc_macro2::Span;

pub trait Spanned {
    fn span(&self) -> Span;
}
