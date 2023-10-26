use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Expr, Ident, ItemFn,
    Lit, LitStr, MetaNameValue, Token,
};

#[proc_macro_attribute]
pub fn devprober(attr: TokenStream, func: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as DevProberAttr);
    let attr_ident = attr.0.iter().map(|(i, _)| i);
    let attr_value = attr.0.iter().map(|(_, v)| v);
    let attr_type = attr.0.iter().map(|(i, _)| match i.to_string().as_str() {
        "compatible" => Ident::new("Compatible", i.span()),
        "device_type" => Ident::new("DeviceType", i.span()),
        "path" => Ident::new("Path", i.span()),
        _ => panic!("invalid attribute"),
    });
    let prober_ident = (0..attr.0.len())
        .map(|i| Ident::new(&format!("__DEV_PROBER_{}", i), Span::call_site().into()));

    let func = parse_macro_input!(func as ItemFn);
    let func_vis = &func.vis;
    let func_name = &func.sig.ident;
    let func_generics = &func.sig.generics;
    let func_block = &func.block;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;

    let caller = quote! {
        #func_vis fn #func_name #func_generics(#func_inputs) #func_output {
            #(
                #[used(linker)]
                #[link_section = concat!(".dev.", stringify!(#attr_ident))]
                static #prober_ident: &jrinx_devprober::DevProber = &jrinx_devprober::DevProber::new(
                    jrinx_devprober::DevIdent::#attr_type(#attr_value),
                    #func_name,
                );
            )*

            #func_block
        }
    };

    caller.into()
}

struct DevProberAttr(Vec<(Ident, LitStr)>);

impl Parse for DevProberAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pairs: Punctuated<MetaNameValue, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(DevProberAttr({
            let mut res = Vec::new();

            for pair in pairs.iter() {
                let ident = pair
                    .path
                    .get_ident()
                    .ok_or(syn::Error::new(pair.path.span(), "ident expected"))?;
                let value: LitStr = {
                    let Expr::Lit(expr_lit) = pair.value.clone() else {
                        return Err(syn::Error::new(pair.value.span(), "lit expected"));
                    };
                    let Lit::Str(lit_str) = expr_lit.lit.clone() else {
                        return Err(syn::Error::new(expr_lit.lit.span(), "str expected"));
                    };
                    lit_str
                };
                res.push((ident.clone(), value));
            }

            res
        }))
    }
}
