//! macro-compose is a library trying to simplify and organize proc-macros.
//! It offers traits ([`Lint`], [`Expand`]) to split up the macro expansion into multiple smaller, reusable parts
//! and structs the collect the results ([`Collector`], [`Context`]).
//!
//! # Example macro
//! The following subsections show examples for different parts of this library.
//!
//! The examples are taken from the example macro in `examples/enum_from_str_macro` which implements a derive macro for `FromStr` for an enum.
//! ## Linting and error handling
//! The [`Lint`] trait is used to lint the macro input. [`Collector::error`] can be used to output errors.
//! ### Example
//! ```
//! # extern crate proc_macro;
//! use macro_compose::{Collector, Lint};
//! use syn::{Data, DeriveInput, Error, Fields};
//!
//! struct EnsureEnumLint;
//!
//! impl Lint<DeriveInput> for EnsureEnumLint {
//!     fn lint(&self, input: &DeriveInput, c: &mut Collector) {
//!         match &input.data {
//!             Data::Enum(e) => {
//!                 for variant in e.variants.iter() {
//!                     if variant.fields != Fields::Unit {
//!                         c.error(Error::new_spanned(&variant.fields, "unexpected fields"))
//!                     }
//!                 }
//!             }
//!             _ => c.error(Error::new_spanned(input, "expected an enum")),
//!         }
//!     }
//! }
//! ```
//! ## Expanding the macro
//! The [`Expand`] trait is used to expand the macro.
//!
//! Once a `Lint` or `Expand` has reported an error to the collector, the macro will no longer be expanded.
//! This way `Expand` implementations can assume that the data checked by `Lint`s is valid.
//! Returning `None` from an `Expand` does **not** automatically report an error.
//! ### Example
//! ```
//! # extern crate proc_macro;
//! use macro_compose::{Collector, Expand};
//! use proc_macro2::Ident;
//! use syn::{parse_quote, Arm, Data, DeriveInput, Error, Fields, ItemImpl};
//! # fn error_struct_ident(_: &DeriveInput) -> Ident {todo!()}
//!
//! struct ImplFromStrExpand;
//!
//! impl Expand<DeriveInput> for ImplFromStrExpand {
//!     type Output = ItemImpl;
//!
//!     fn expand(&self, input: &DeriveInput, _: &mut Collector) -> Option<Self::Output> {
//!         let variants = match &input.data {
//!             Data::Enum(e) => &e.variants,
//!             _ => unreachable!(),
//!         };
//!         let ident = &input.ident;
//!
//!         let arms = variants.iter().map(|v| -> Arm {
//!             let v = &v.ident;
//!             let name = v.to_string();
//!             parse_quote!(
//!                 #name => ::core::result::Result::Ok(#ident :: #v)
//!             )
//!         });
//!
//!         let ident = &input.ident;
//!         let error = error_struct_ident(input);
//!         Some(parse_quote!(
//!             impl ::core::str::FromStr for #ident {
//!                 type Err = #error;
//!
//!                 fn from_str(s: &::core::primitive::str) -> ::core::result::Result<Self, Self::Err> {
//!                     match s {
//!                         #(#arms,)*
//!                         invalid => ::core::result::Result::Err( #error (::std::string::ToString::to_string(invalid))),
//!                     }
//!                 }
//!             }
//!         ))
//!     }
//! }
//! ```
//! ## Implementing the macro
//! [`Context::new_parse`] can be used to create a context from a [`TokenStream`](proc_macro::TokenStream).
//! This Context can be used to run `Lint`s and `Expand`s and get the resulting output.
//! ### Example
//! ```
//! # extern crate proc_macro;
//! use macro_compose::{Collector, Context};
//! use proc_macro::TokenStream;
//!
//! # #[doc = "
//! #[proc_macro_derive(FromStr)]
//! pub fn derive_from_str(item: TokenStream) -> TokenStream {
//!     let mut collector = Collector::new();
//!
//!     let mut ctx = Context::new_parse(&mut collector, item);
//!     ctx.lint(&EnsureEnumLint);
//!
//!     ctx.expand(&ErrorStructExpand);
//!     ctx.expand(&ImplDebugErrorStructExpand);
//!     ctx.expand(&ImplFromStrExpand);
//!
//!     collector.finish()
//! }
//! # "]
//! # struct Foo;
//! ```

#![deny(missing_docs, clippy::doc_markdown)]

extern crate proc_macro;

mod context;

pub use context::{Collector, Context};

use proc_macro2::TokenStream;
use quote::ToTokens;

/// Lint is used for linting the macro input
///
/// # Example
/// ```
/// use macro_compose::{Collector, Lint};
/// use syn::{Data, DeriveInput, Error};
///
/// struct EnsureStructLint;
///
/// impl Lint<DeriveInput> for EnsureStructLint {
///     fn lint(&self, input: &DeriveInput, c: &mut Collector) {
///         if !matches!(&input.data, Data::Struct(_)) {
///             c.error(Error::new_spanned(input, "expected a struct"));
///         }
///     }
/// }
/// ```
pub trait Lint<I> {
    /// lint the macro input
    fn lint(&self, input: &I, c: &mut Collector);
}

/// Expand is used for expanding macros
///
/// # Example
/// ```
/// use macro_compose::{Collector, Expand};
/// use syn::{parse_quote, DeriveInput, Error, ItemImpl};
///
/// struct ImplFooExpand;
///
/// impl Expand<DeriveInput> for ImplFooExpand {
///     type Output = ItemImpl;
///     
///     fn expand(&self, input: &DeriveInput, c: &mut Collector) -> Option<Self::Output> {
///         let ident = &input.ident;
///         
///         Some(parse_quote!(
///             impl Foo for #ident {
///                 fn bar(&self) {}
///             }   
///         ))
///     }
/// }
/// ```
pub trait Expand<I> {
    /// the output generated by the expansion
    type Output: ToTokens;

    /// expand the macro
    fn expand(&self, input: &I, c: &mut Collector) -> Option<Self::Output>;
}

/// a helper struct for expanding to nothing
pub struct Nothing;

impl ToTokens for Nothing {
    fn to_tokens(&self, _: &mut TokenStream) {}
}

/// simply echo the input
pub struct EchoExpand;

impl<T: ToTokens + Clone> Expand<T> for EchoExpand {
    type Output = T;

    fn expand(&self, input: &T, _: &mut Collector) -> Option<Self::Output> {
        Some(input.clone())
    }
}
