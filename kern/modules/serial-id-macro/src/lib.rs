use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

const NOTE: &str = "SerialId can only be derived for tuple structs with one field";

#[proc_macro_derive(SerialId)]
pub fn serial_id(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let expand = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(ref f),
            ..
        }) if f.unnamed.len() == 1 => {
            let ty = &f.unnamed.first().unwrap().ty;
            Ok(quote! {
                impl #name {
                    fn new() -> Self {
                        static ID_GENERATOR: spin::Mutex<#ty> = spin::Mutex::new(0);

                        let mut id = ID_GENERATOR.lock();
                        *id = id.checked_add(1).unwrap();
                        Self(*id)
                    }
                }
            })
        }
        _ => Err(NOTE),
    }
    .unwrap();

    expand.into()
}
