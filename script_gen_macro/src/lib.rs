extern crate proc_macro;

use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};
use proc_macro::TokenStream;

#[proc_macro_derive(ScriptComponentUpdater, attributes(SyncComponent))]
pub fn derive_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let struct_name = ast.ident.clone();
    let attribute_fields = match ast {
        syn::DeriveInput {
            data: syn::Data::Struct(
                syn::DataStruct {
                    fields: syn::Fields::Named(
                        syn::FieldsNamed {
                            named: fields,
                            ..
                        }
                    ), ..
                }
            ), ..
        } => fields,
        _ => unimplemented!("No")
    };

    let filtered_fields: Vec<&syn::Field> = attribute_fields
        .iter()
        .filter(|field| {
            for attribute in field.attrs.iter() {
                if attribute.path().get_ident().unwrap() == "SyncComponent" {
                    return true;
                }
            }
            false
        })
        .collect();

    let register_calls = filtered_fields
        .iter()
        .map(|field| {
            let cl_field = (**field).clone();
            let variable = cl_field.ident.unwrap();
            let ty = cl_field.ty;

            quote! {
                world.register_component::<#ty>(&self.entity, self.#variable.clone());
            }
        });

    let pre_user_update_calls = filtered_fields
        .iter()
        .map(|field| {
            let cl_field = (**field).clone();
            let variable = cl_field.ident.unwrap();
            let ty = cl_field.ty;

            quote! {
                if let Some(c) = world.get_entity_component::<#ty>(&self.entity) {
                    self.#variable = c.clone()
                }
            }
        });

    let post_user_update_calls = filtered_fields
        .iter()
        .map(|field| {
            let cl_field = (**field).clone();
            let variable = cl_field.ident.unwrap();
            let ty = cl_field.ty;

            quote! {
                if let Some(c) = world.get_entity_component_mut::<#ty>(&self.entity) {
                    *c = self.#variable.clone()
                }
            }
        });


    let output = quote! {
        impl ScriptComponentUpdater for #struct_name {
            fn pre_setup(&mut self, entity: probable_spork_ecs::component::Entity, world: &mut probable_spork_ecs::world::World) {
                #(#register_calls)*
            }

            fn pre_user_update(&mut self, world: &probable_spork_ecs::world::World) {
                #(#pre_user_update_calls)*
            }
            fn post_user_update(&mut self, world: &mut probable_spork_ecs::world::World) {
                #(#post_user_update_calls)*
            }
        }
    };

    TokenStream::from(output)
}