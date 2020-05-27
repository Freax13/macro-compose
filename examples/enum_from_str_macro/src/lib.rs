extern crate proc_macro;
use macro_compose::{Collector, Context, Expand, Lint};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::format_ident;
use syn::{parse_quote, Arm, Data, DeriveInput, Error, Fields, ItemImpl, ItemStruct};

#[proc_macro_derive(FromStr)]
pub fn derive_from_str(item: TokenStream) -> TokenStream {
    let mut collector = Collector::new();

    let mut ctx = Context::new_parse(&mut collector, item);
    ctx.lint(&EnsureEnumLint);

    ctx.expand(&ErrorStructExpand);
    ctx.expand(&ImplDebugErrorStructExpand);
    ctx.expand(&ImplFromStrExpand);

    collector.finish()
}

struct EnsureEnumLint;

impl Lint<DeriveInput> for EnsureEnumLint {
    fn lint(&self, input: &DeriveInput, c: &mut Collector) {
        match &input.data {
            Data::Enum(e) => {
                for variant in e.variants.iter() {
                    if variant.fields != Fields::Unit {
                        c.error(Error::new_spanned(&variant.fields, "unexpected fields"))
                    }
                }
            }
            _ => c.error(Error::new_spanned(input, "expected an enum")),
        }
    }
}

fn error_struct_ident(input: &DeriveInput) -> Ident {
    format_ident!("Parse{}Error", &input.ident)
}

struct ErrorStructExpand;

impl Expand<DeriveInput> for ErrorStructExpand {
    type Output = ItemStruct;

    fn expand(&self, input: &DeriveInput, _: &mut Collector) -> Option<Self::Output> {
        let ident = error_struct_ident(input);
        let vis = &input.vis;
        Some(parse_quote!(
            #[derive(Clone, PartialEq, Eq)]
            #vis struct #ident (::std::string::String);
        ))
    }
}

struct ImplDebugErrorStructExpand;

impl Expand<DeriveInput> for ImplDebugErrorStructExpand {
    type Output = ItemImpl;

    fn expand(&self, input: &DeriveInput, _: &mut Collector) -> Option<Self::Output> {
        let ident = error_struct_ident(input);
        let ident_name = ident.to_string();
        Some(parse_quote!(
            impl ::core::fmt::Debug for #ident {
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    write!(f, "unexpected value for {}: {}", #ident_name, &self.0)
                }
            }
        ))
    }
}

struct ImplFromStrExpand;

impl Expand<DeriveInput> for ImplFromStrExpand {
    type Output = ItemImpl;

    fn expand(&self, input: &DeriveInput, _: &mut Collector) -> Option<Self::Output> {
        let variants = match &input.data {
            Data::Enum(e) => &e.variants,
            _ => unreachable!(),
        };
        let ident = &input.ident;

        let arms = variants.iter().map(|v| -> Arm {
            let v = &v.ident;
            let name = v.to_string();
            parse_quote!(
                #name => ::core::result::Result::Ok(#ident :: #v)
            )
        });

        let ident = &input.ident;
        let error = error_struct_ident(input);
        Some(parse_quote!(
            impl ::core::str::FromStr for #ident {
                type Err = #error;

                fn from_str(s: &::core::primitive::str) -> ::core::result::Result<Self, Self::Err> {
                    match s {
                        #(#arms,)*
                        invalid => ::core::result::Result::Err( #error (::std::string::ToString::to_string(invalid))),
                    }
                }
            }
        ))
    }
}
