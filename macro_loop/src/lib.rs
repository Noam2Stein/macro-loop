#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod expr;
mod fragment;
mod name;
mod value;

mod util;
use util::*;

#[cfg(test)]
mod speedtests;

/// `macro_loop!` provides special fragment features using `@`.
///
/// # For Loops
///
/// Syntax: `@for <item> in <values> { ... }`
///
/// For loops emit their body per value:
///
/// ```rust
/// macro_loop! {
///     @for N in 2..=4 {
///         struct @[Vec @N];
///     }
/// }
///
/// // outputs:
/// // struct Vec2;
/// // struct Vec3;
/// // struct Vec4;
/// ```
///
/// The `<item>` needs to be a ***pattern*** - Either:
/// * an identifier (`Prime`),
/// * an array of patterns (`[CapeLight, [Sprinter, Truth]]`).
///
/// The `<values>` needs to be an array value that matches the `<item>` pattern.
/// A values is either:
/// * a literal,
/// * an identifier,
/// * an array of values.
///
/// Values support operators such as `+`, `..` and `==`.
///
/// Declaring a for loop with multiple parameters (`@for a in [...], b in [...]`),
/// emits the body per value combination.
///
/// # If Statements
///
/// Syntax: `@if <condition> { ... }`
///
/// An if statement emits its body only if its condition is met:
///
/// ```rust
/// macro_rules! not_equal {
///     ($a:ident $b:ident) => {
///         macro_loop! {
///             @if $a != $b {
///                 println!("{}", stringify!($a != $b))
///             }
///         }
///     };
/// }
///
/// fn main() {
///     not_equal!(a a); // doesn't print
///     not_equal!(a b); // prints
///     not_equal!(b b); // doesn't print
/// }
/// ```
///
/// The `<condition>` needs to be a bool value.
///
/// # Let Statements
///
/// Syntax: `@let <name> = <value>;`
///
/// Let statements declare names that have a value:
///
/// macro_loop! {
///     @let components = [x, y, z, w];
///
///     @for X in @components, Y in @components] {
///         ...
///     }
/// }
///
/// The `<name>` needs to be a pattern, and the ~value~ has to match it.
///
/// # Identifiers
///
/// Syntax: `@[<idents>]`
///
/// Concats identifier segments into a single identifier:
///
/// ```rust
/// @let N = 5;
///
/// struct @[Struct @N]; // Struct5
/// ```
#[proc_macro]
pub fn macro_loop(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use syn::parse::Parser;

    match macro_loop2.parse2(input.into()) {
        Ok(stream) => stream,
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn macro_loop2(input: syn::parse::ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    use syn::parse::Parse;

    let name_stream = name::NameStream::parse(input)?;

    name_stream.resolve(&name::Namespace::new())
}
