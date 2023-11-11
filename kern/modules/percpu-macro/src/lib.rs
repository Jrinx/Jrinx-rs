use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Error, Ident, ItemStatic, StaticMutability};

#[proc_macro_attribute]
pub fn percpu(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new(Span::call_site().into(), "empty attributes expected")
            .to_compile_error()
            .into();
    }

    let ast = parse_macro_input!(item as ItemStatic);
    let attrs = &ast.attrs;
    let vis = &ast.vis;
    let mutability = &ast.mutability;
    let name = &ast.ident;
    let ty = &ast.ty;
    let expr = &ast.expr;

    let inner_name = &Ident::new(&format!("__PERCPU_{}", name), Span::call_site().into());
    let struct_name = &Ident::new(
        &format!("{}_PERCPU_WRAPPER", name),
        Span::call_site().into(),
    );
    let iter_name = &Ident::new(&format!("{}_PERCPU_ITER", name), Span::call_site().into());

    let safety = match mutability {
        StaticMutability::None => quote!(),
        _ => quote!(unsafe),
    };

    let expand = quote! {
        #[allow(non_camel_case_types)]
        #vis struct #struct_name {}

        #[allow(non_camel_case_types)]
        #vis struct #iter_name {
            cpu_id: usize,
        }

        #[link_section = ".percpu"]
        #(#attrs)*
        static #mutability #inner_name: #ty = #expr;

        #(#attrs)*
        #vis static #name: #struct_name = #struct_name {};

        impl #struct_name {
            #[inline]
            pub fn offset(&self) -> usize {
                unsafe { &#inner_name as *const _ as usize - jrinx_layout::_spercpu() }
            }

            #[inline]
            pub #safety fn as_ptr(&self) -> *const #ty {
                (jrinx_percpu::get_local_pointer() + self.offset()) as *const #ty
            }

            #[inline]
            pub #safety fn as_ref(&self) -> &#ty {
                unsafe { &*self.as_ptr() }
            }

            #[inline]
            pub #safety fn with_ref<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&#ty) -> R,
            {
                f(self.as_ref())
            }

            #[inline]
            pub #safety fn with_spec_ref<F, R>(&self, cpu_id: usize, f: F) -> R
            where
                F: FnOnce(&#ty) -> R,
            {
                f(unsafe {
                    &*((jrinx_percpu::local_area_base(cpu_id) + self.offset()) as *const #ty)
                })
            }

            #[allow(clippy::mut_from_ref)]
            #[inline]
            pub #safety fn as_ref_mut(&self) -> &mut #ty {
                unsafe { &mut *(self.as_ptr() as *mut #ty) }
            }

            #[inline]
            pub #safety fn with_ref_mut<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&mut #ty) -> R,
            {
                f(self.as_ref_mut())
            }

            #[inline]
            pub #safety fn with_spec_ref_mut<F, R>(&self, cpu_id: usize, f: F) -> R
            where
                F: FnOnce(&mut #ty) -> R,
            {
                f(unsafe {
                    &mut *((jrinx_percpu::local_area_base(cpu_id) + self.offset()) as *mut #ty)
                })
            }

            #[inline]
            pub #safety fn iter(&self) -> #iter_name {
                #iter_name {
                    cpu_id: 0,
                }
            }
        }

        impl core::iter::Iterator for #iter_name {
            type Item = &'static #ty;

            fn next(&mut self) -> Option<Self::Item> {
                use jrinx_hal::{Cpu, Hal};

                if self.cpu_id >= jrinx_hal::hal!().cpu().nproc() {
                    None
                } else {
                    let cpu_id = self.cpu_id;
                    self.cpu_id += 1;
                    Some(unsafe {
                        &*((jrinx_percpu::local_area_base(cpu_id) + #name.offset()) as *const #ty)
                    })
                }
            }
        }
    };

    expand.into()
}
