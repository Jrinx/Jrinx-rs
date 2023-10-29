use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, Fields};

const NOTE: &str = "Address can only be derived for tuple structs with one field";

#[proc_macro_derive(Address)]
pub fn address(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);
    let name = &input.ident;

    let expand = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(ref f),
            ..
        }) if f.unnamed.len() == 1 => {
            let ty = &f.unnamed.first().unwrap().ty;
            Ok(quote! {
                impl core::ops::Add<#ty> for #name {
                    type Output = Self;

                    fn add(self, rhs: #ty) -> Self::Output {
                        Self(self.0 + rhs)
                    }
                }

                impl core::ops::Sub<#ty> for #name {
                    type Output = Self;

                    fn sub(self, rhs: #ty) -> Self::Output {
                        Self(self.0 - rhs)
                    }
                }

                impl core::ops::Sub<#name> for #name {
                    type Output = #ty;

                    fn sub(self, rhs: #name) -> Self::Output {
                        self.0 - rhs.0
                    }
                }

                impl core::fmt::Display for #name {
                    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(f, "0x{:x}", self.0)
                    }
                }

                impl #name {
                    pub const fn new(addr: #ty) -> Self {
                        Self(addr)
                    }

                    pub const fn as_usize(self) -> #ty {
                        self.0
                    }

                    pub fn align_page_down(self) -> Self {
                        Self(self.0 & !(jrinx_config::PAGE_SIZE - 1))
                    }

                    pub fn align_page_up(self) -> Self {
                        Self((self.0 + jrinx_config::PAGE_SIZE - 1) & !(jrinx_config::PAGE_SIZE - 1))
                    }
                }
            })
        }
        _ => Err(NOTE),
    }
    .unwrap();

    expand.into()
}
