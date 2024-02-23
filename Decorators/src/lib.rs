extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, ItemStruct,
};



#[proc_macro_derive(Signaler, attributes(property))]
pub fn derive_decorator(input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = item_struct.ident;

    let mut properties: Vec<(proc_macro2::Ident, syn::Type)> = vec![];
    let opt_decs: Vec<(proc_macro2::Ident, syn::Type)> = vec![];

    if let syn::Fields::Named(ref fields_named) = item_struct.fields {
        for field in fields_named.named.iter() {
            for attr in field.attrs.iter() {
                if attr.path().is_ident("property") {
                    let item = field.clone();
                    properties.push((item.ident.unwrap(), item.ty))
                }
            }
        }
    }

    let signals_def = properties.iter().fold(quote!(), |_acc, (name, ty)| {
        let signal_name = format_ident!("signal_{name}");
        quote! {
            #signal_name: Signal<#ty>,
        }
    });

    let functions = properties.iter().fold(quote!(), |acc, (name, ty)| {
        let on_name = format_ident!("on_{name}_changed");
        let emit_name = format_ident!("emit_{name}");
        let signal_name = format_ident!("signal_{name}");
        let set_name = format_ident!("set_{name}");
        quote! {
            #acc
            pub fn #name(&self) -> #ty {
                self.data.#name.clone()
            }

            #acc
            pub fn #set_name(&mut self, value: #ty) {
                self.data.#name = value;
                self.#emit_name()
            }

            #acc
            pub fn #on_name(&self) -> &Signal<#ty> {
                &self.#signal_name
            }

            #acc
            pub fn #emit_name(&self) {
                self.#signal_name.emit(self.data.#name.clone());
            }
        }
    });

    let signals_new = properties.iter().fold(quote!(), |_acc, (name, _ty)| {
        let signal_name = format_ident!("signal_{name}");
        quote! {
            #signal_name: Signal::new(),
        }
    });

    let opt_decs = opt_decs.iter().fold(quote!(), |acc, (name, ty)| {
        quote! {
            #acc
            pub fn #name(self, #name:#ty ) -> Self {
                Self {
                    #name: Some(#name),
                    ..self
                }
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    let signaler_object_name = format_ident!("{struct_name}Signaler");
    let k = quote! {
        struct #signaler_object_name {
            data: #struct_name,

            #signals_def
        }

        impl #impl_generics #signaler_object_name #ty_generics #where_clause  {
            pub fn new() -> Self {
                Self {
                    data: #struct_name::default(),
                    #signals_new
                }
            }
            #functions

            #opt_decs
        }
    };

    dbg!(&k.to_string());

    TokenStream::from(k)
}
