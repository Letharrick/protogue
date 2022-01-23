extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate proc_macro2;

use quote::ToTokens;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Type, Field, PathArguments, Ident};
use quote::{quote, format_ident};
use proc_macro2::Span;

fn field_to_component_code(command_buf_ident: &Ident, entity_ident: &Ident, field: &Field) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let field_name = &field.ident;

    match &field.ty {
        Type::Path(type_path) => {
            let first_segment = type_path.path.segments.first().unwrap();

            return if type_path.path.leading_colon.is_none() {
                match first_segment.ident.to_string().as_str() {
                    // If the component is optional,
                    stringify!(Option) => {
                        match &first_segment.arguments {
                            PathArguments::AngleBracketed(args) => {
                                let component_type = args.args.first().unwrap().to_token_stream();

                                // Return a 2-tuple that contains that type in the option and the
                                (
                                    component_type.clone(),
                                    quote! {
                                        if self.#field_name.is_some() {
                                            #command_buf_ident.add_component(
                                                #entity_ident,
                                                self.#field_name.as_ref().map(|component| component.clone()).unwrap()
                                            );
                                        }
                                    }
                                )
                            },
                            _ => panic!()
                        }
                    },

                    _ => (first_segment.ident.to_token_stream(), quote! {
                        #command_buf_ident.add_component(
                            #entity_ident,
                            self.#field_name.clone()
                        );
                    })
                }
            } else {
                panic!()
            };
        },
        _ => panic!()
    }
}

#[proc_macro_derive(ObjectBase)]
pub fn object_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;
    let command_buf_ident = Ident::new("commands", Span::call_site());
    let entity_ident = Ident::new("entity", Span::call_site());

    // If the macro is on a struct
    if let Data::Struct(data) = &input.data {
        let mut blocks_fov = false;
        let mut blocks_movement = false;

        let component_type_and_code = data.fields.iter().map(|field| {
            field_to_component_code(&command_buf_ident, &entity_ident, field)
        }).collect::<Vec<_>>();

        // Use components to infer FOV-blocking and movement-blocking
        for (component_type, code) in &component_type_and_code {
            match component_type.to_string().as_str() {
                stringify!(Barrier) => blocks_movement = true,
                stringify!(Opaque) => blocks_fov = true,
                _ => {}
            }
        }

        let component_add_code = component_type_and_code.iter().cloned().map(|(component_type, code)| code);

        // Write an impl for the object
        let implementation = quote! {
            impl #struct_name {
                // A method for spawning this object in the world
                pub fn spawn<P: Copy + Into<Vector<i32>>>(&self, #command_buf_ident: &mut CommandBuffer, map: Arc<RwLock<Map>>, position: P) -> legion::Entity {
                    let #entity_ident = #command_buf_ident.push((Position { vector: position.into() },));

                    #(
                        #component_add_code;
                    )*


                    if let Ok(mut map) = map.write() {
                        map[position.into()].push(
                            Object::new(#entity_ident, #blocks_fov, #blocks_movement)
                        );
                    }

                    #entity_ident
                }
            }
        };

        return TokenStream::from(implementation)
    }

    TokenStream::default()
}