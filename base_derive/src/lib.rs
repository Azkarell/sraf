
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};



#[proc_macro_derive(Resource)]
pub fn resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote!{

        impl #impl_generics ::app_base::storage::Resource for #name #ty_generics #where_clause {
            fn id() -> ::app_base::Uuid {
                ::app_base::Uuid::new_v5(&::app_base::Uuid::NAMESPACE_OID, stringify!(#name).as_bytes())
            }

            fn as_any<'a>(&'a self) -> &'a dyn std::any::Any {
                self
            }

            fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn std::any::Any {
                self
            }
        }
    }.into()
}

#[proc_macro_derive(Event)]
pub fn event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote!{
        impl #impl_generics ::app_base::Event for #name #ty_generics #where_clause {
            fn event_id() -> ::app_base::Uuid {
                ::app_base::Uuid::new_v5(&::app_base::Uuid::NAMESPACE_OID, stringify!(#name).as_bytes())
            }

        }
    }.into()
}