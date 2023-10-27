use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn testdef(_: TokenStream, func: TokenStream) -> TokenStream {
    let func = parse_macro_input!(func as ItemFn);
    let func_vis = &func.vis;
    let func_name = &func.sig.ident;
    let func_generics = &func.sig.generics;
    let func_block = &func.block;
    let func_inputs = &func.sig.inputs;
    let func_output = &func.sig.output;

    let caller = quote! {
        #func_vis fn #func_name #func_generics(#func_inputs) #func_output {
            #[cfg_attr(feature = "no_test", used)]
            #[cfg_attr(
                not(feature = "no_test"),
                used(linker),
                link_section = concat!(".test.", module_path!()),
            )]
            static __TEST_DEF: &jrinx_testdef::TestDef = &jrinx_testdef::TestDef::new(
                module_path!(),
                #func_name,
            );

            #func_block
        }
    };

    caller.into()
}
