use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{DeriveInput, Field, Type};

pub struct ScriptComponentUpdaterMacro;

impl ScriptComponentUpdaterMacro {
    fn script_fn_closure<'a, F: Fn(Type, Ident) -> TokenStream + 'a>(filtered_fields: &'a Vec<&'a Field>, closure: F) -> impl Iterator<Item = TokenStream> + 'a {
        let register_calls = filtered_fields
            .iter()
            .map(move |field| -> TokenStream {
                let cl_field = (**field).clone();
                let variable = cl_field.ident.unwrap();
                let ty = cl_field.ty;


                (closure)(ty, variable)
            });
        return register_calls;
    }

    pub fn generate_output(ast: DeriveInput) -> TokenStream {
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
            _ => unimplemented!("Couldn't find any fields on struct")
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

        let register_calls = ScriptComponentUpdaterMacro::script_fn_closure(&filtered_fields, |ty, variable| {
            quote! {
                world.register_component::<#ty>(&self.entity, self.#variable.clone());
            }
        });

        let pre_user_update_calls = ScriptComponentUpdaterMacro::script_fn_closure(&filtered_fields, |ty, variable| {
            quote! {
                if let Some(c) = world.get_entity_component::<#ty>(&self.entity) {
                    if self.#variable != *c {
                        self.#variable = c.clone()
                    }
                }
            }
        });

        let post_user_update_calls = ScriptComponentUpdaterMacro::script_fn_closure(&filtered_fields, |ty, variable| {
            quote! {
                if let Some(mut c) = world.get_entity_component_mut::<#ty>(&self.entity) {
                    if *c != self.#variable {
                        *c = self.#variable.clone();
                    }
                }
            }
        });


        quote! {
            impl ScriptComponentUpdater for #struct_name {
                fn pre_setup(&mut self, entity: probable_spork_ecs::component::Entity, world: &mut probable_spork_ecs::component::ComponentStorage) {
                    #(#register_calls)*
                }

                fn pre_user_update(&mut self, world: &probable_spork_ecs::component::ComponentStorage) {
                    #(#pre_user_update_calls)*
                }
                fn post_user_update(&mut self, world: &probable_spork_ecs::component::ComponentStorage) {
                    #(#post_user_update_calls)*
                }
            }
        }
    }
}
