extern crate proc_macro;

use proc_macro2::{Ident, Literal};
use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput, Data, Path, Variant, Expr};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::Fields::{Named, Unnamed};

/// Example of user-defined [derive mode macro][1]
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
#[proc_macro_derive(SwapEndian)]
pub fn swap_endian(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    let name = ast.ident;

    let mut swapped_content = proc_macro2::TokenStream::new();

    match ast.data {
        Data::Struct(data) => {
            match data.fields {
                Named(named) => {
                    let mut inner = proc_macro2::TokenStream::new();

                    for field in named.named.iter() {
                        let name = field.ident.as_ref().unwrap().clone();


                        inner.append_all(quote! {#name: crate::endianness::SwapEndian::swap_endian(self.#name),});
                    }

                    swapped_content.append_all(quote! {{#inner}});
                }
                Unnamed(unnamed) => {
                    let mut inner = proc_macro2::TokenStream::new();

                    for (idx, field) in unnamed.unnamed.iter().enumerate() {
                        let literal = Literal::usize_unsuffixed(idx);
                        inner.append_all(quote! {crate::endianness::SwapEndian::swap_endian(self.#literal),});
                    }

                    swapped_content.append_all(quote! {(#inner)});
                }
                _ => { swapped_content.append_all(quote! {()}) }
            }
        }
        _ => unimplemented!()
    }

    quote! {
        impl crate::endianness::SwapEndian for #name{
            fn swap_endian(self) -> Self {
                Self #swapped_content
            }
        }
    }.into()
}

fn get_descriminant<'a>(iter: &mut impl Iterator<Item = &'a Variant>) -> Option<Expr>{
    Some(iter.next()
        .as_ref()?
        .discriminant
        .as_ref()
        .expect("every enum variant must have a descriminant")
        .1
        .clone())
}

#[proc_macro_derive(EnumTryInto)]
pub fn enum_try_into(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    let impl_on_name = ast.ident;

    let Data::Enum(enum_data) = ast.data else {
        panic!("must be used on an enum");
    };

    let repr_attr = ast.attrs.iter()
        .find(|a| a.path.segments.iter().next().unwrap().ident.clone().into_token_stream().to_string() == "repr")
        .expect("no repr found, you need to specify a repr for this to work");

    let repr_type = repr_attr.tokens.clone();

    let mut iter = enum_data.variants.iter();

    let mut first_match_content = get_descriminant(&mut iter)
        .expect("at least one enum variant must exist");

    let mut match_content = quote!{
        #first_match_content
    };

    while let Some(descr) = get_descriminant(&mut iter){
        match_content.append_all(quote!{
            | #descr
        })
    }





    quote!{
        impl ::core::convert::TryFrom<u32> for #impl_on_name{
            type Error = ();

            fn try_from(value: u32) -> ::core::result::Result<Self, Self::Error> {
                match(value){
                    #match_content => Ok(unsafe{ std::mem::transmute(value) }),
                    _ => Err(())
                }
            }
        }

    }.into()
}
