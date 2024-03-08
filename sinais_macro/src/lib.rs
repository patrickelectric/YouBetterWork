extern crate proc_macro2;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};

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

    let signals_def = properties.iter().fold(quote!(), |acc, (name, ty)| {
        let signal_name = format_ident!("signal_{name}");
        let signal_inner_name = format_ident!("signal_inner_{name}");
        quote! {
            #acc
            #signal_name: Signal<#ty>,
            #signal_inner_name: SignalInner<Self, #ty>,
        }
    });

    let mut all_properties_emit = vec![];
    let functions = properties.iter().fold(quote!(), |acc, (name, ty)| {
        let on_name = format_ident!("on_{name}_changed");
        let emit_name = format_ident!("emit_{name}");
        let signal_name = format_ident!("signal_{name}");
        let signal_inner_name = format_ident!("signal_inner_{name}");
        let on_inner_name_changed = format_ident!("on_inner_{name}_changed");
        let set_name = format_ident!("set_{name}");

        all_properties_emit.push(emit_name.clone());

        quote! {
            #acc

            pub fn #name(&self) -> #ty {
                self.data.#name.clone()
            }

            pub fn #set_name(&mut self, value: #ty) {
                self.data.#name = value;
                self.#emit_name();
            }

            pub fn #on_name(&self) -> &Signal<#ty> {
                &self.#signal_name
            }

            pub fn #on_inner_name_changed(&mut self, function: fn(&mut Self, #ty)) {
                self.#signal_inner_name.add(function);
            }

            pub fn #emit_name(&mut self) {
                if (self.#signal_inner_name.calls.len() > 0) {
                    let mut calls = std::mem::replace(&mut self.#signal_inner_name.calls, Vec::new());

                    for call in calls.iter_mut() {
                        call(self, self.data.#name.clone());
                    }

                    self.#signal_inner_name.calls = calls;
                }
                self.#signal_name.emit(self.data.#name.clone());
            }
        }
    });

    let all_properties_emit = all_properties_emit.iter().fold(quote!(), |acc, emit_name| {
        quote! {
            #acc
            self.#emit_name();
        }
    });
    let functions = quote! {
        #functions

        pub fn emit_all_properties(&mut self) {
            #all_properties_emit
        }

        /*
        pub fn on_self_changed(&self) -> &Signal<#struct_name> {
            &self.self_signal
        }

        pub fn emit(&self) {
            self.self_signal.emit(self.data.clone())
        }
        */
    };

    let signals_new = properties.iter().fold(quote!(), |acc, (name, _ty)| {
        let signal_name = format_ident!("signal_{name}");
        let signal_inner_name = format_ident!("signal_inner_{name}");
        quote! {
            #acc
            #signal_name: Signal::new(),
            #signal_inner_name: SignalInner::new(),
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

            // self_signal: Signal<#struct_name>,

            #signals_def
        }

        impl Default for #signaler_object_name {
            fn default() -> Self {
                Self {
                    data: Default::default(),
                    #signals_new
                }
            }
        }

        impl #impl_generics #signaler_object_name #ty_generics #where_clause  {
            #functions

            #opt_decs
        }
    };

    // dbg!(k.to_string());

    TokenStream::from(k)
}
