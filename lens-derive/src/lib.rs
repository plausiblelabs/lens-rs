//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Visibility};

/// Handles the `#derive(Lenses)` applied to a struct by generating a `Lens` implementation for
/// each field in the struct.
#[proc_macro_derive(Lenses)]
pub fn lenses_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Check that the input type is a struct
    let data_struct: DataStruct;
    if let Data::Struct(s) = input.data {
        data_struct = s
    } else {
        panic!("`#[derive(Lenses)]` may only be applied to structs")
    }

    // Check that the struct has named fields, since that's the only
    // type we support at the moment
    let fields: Fields;
    if let Fields::Named(_) = data_struct.fields {
        fields = data_struct.fields
    } else {
        panic!("`#[derive(Lenses)]` may only be applied to structs with named fields")
    }

    // Extract the struct name
    let struct_name = &input.ident;

    // Determine the visibility of the lens struct
    let lens_visibility = match input.vis {
        Visibility::Public(..) => quote!(pub),
        // TODO: Handle `Crate` and `Restricted` visibliity
        Visibility::Crate(..) => quote!(),
        Visibility::Restricted(..) => quote!(),
        Visibility::Inherited => quote!(),
    };

    // Generate lenses for each field in the struct
    let lens_items = fields.iter().enumerate().map(|(index, field)| {
        if let Some(field_name) = &field.ident {
            let field_index = index as u64;
            let field_type = &field.ty;

            // Build the Lens name from the struct name and field name (for example, "StructFieldLens")
            let lens_name = format_ident!(
                "{}{}Lens",
                struct_name.to_string(),
                to_camel_case(&field_name.to_string())
            );

            // Build a `ValueLens` impl if the target is a primitive
            // TODO: Should do this automatically for any target type that implements `Clone`
            let value_lens = if is_primitive(&field.ty) {
                quote!(
                    #[allow(dead_code)]
                    impl pl_lens::ValueLens for #lens_name {
                        #[inline(always)]
                        fn get(&self, source: &#struct_name) -> #field_type {
                            (*source).#field_name.clone()
                        }
                    }
                )
            } else {
                quote!()
            };

            quote!(
                // Include the lens struct declaration
                #[allow(dead_code)]
                #[doc(hidden)]
                #lens_visibility struct #lens_name;

                // Include the `Lens` impl
                #[allow(dead_code)]
                impl pl_lens::Lens for #lens_name {
                    type Source = #struct_name;
                    type Target = #field_type;

                    #[inline(always)]
                    fn path(&self) -> pl_lens::LensPath {
                        pl_lens::LensPath::new(#field_index)
                    }

                    #[inline(always)]
                    fn mutate<'a>(&self, source: &'a mut #struct_name, target: #field_type) {
                        source.#field_name = target
                    }
                }

                // Include the `RefLens` impl
                #[allow(dead_code)]
                impl pl_lens::RefLens for #lens_name {
                    #[inline(always)]
                    fn get_ref<'a>(&self, source: &'a #struct_name) -> &'a #field_type {
                        &(*source).#field_name
                    }

                    #[inline(always)]
                    fn get_mut_ref<'a>(&self, source: &'a mut #struct_name) -> &'a mut #field_type {
                        &mut (*source).#field_name
                    }
                }

                // Include the `ValueLens` impl (only if it should be defined)
                #value_lens
            )
        } else {
            // This should be unreachable, since we already verified above that the struct
            // only contains named fields
            panic!("`#[derive(Lenses)]` may only be applied to structs with named fields")
        }
    });

    // Build a `<StructName>Lenses` struct that enumerates the available lenses
    // for each field in the struct, for example:
    //     struct Struct2Lenses {
    //         int32: Struct2Int32Lens,
    //         struct1: Struct2Struct1Lens,
    //         struct1_lenses: Struct1Lenses
    //     }
    let lenses_struct_name = format_ident!("{}Lenses", struct_name);
    let lenses_struct_fields = fields.iter().map(|field| {
        if let Some(field_name) = &field.ident {
            let field_lens_name = format_ident!(
                "{}{}Lens",
                struct_name,
                to_camel_case(&field_name.to_string())
            );
            if is_primitive(&field.ty) {
                quote!(#field_name: #field_lens_name)
            } else {
                let field_parent_lenses_field_name = format_ident!("{}_lenses", field_name);
                let field_parent_lenses_type_name =
                    format_ident!("{}Lenses", to_camel_case(&field_name.to_string()));
                quote!(
                    #field_name: #field_lens_name,
                    #field_parent_lenses_field_name: #field_parent_lenses_type_name
                )
            }
        } else {
            // This should be unreachable, since we already verified above that the struct
            // only contains named fields
            panic!("`#[derive(Lenses)]` may only be applied to structs with named fields")
        }
    });
    let lenses_struct = quote!(
        #[allow(dead_code)]
        #[doc(hidden)]
        #lens_visibility struct #lenses_struct_name {
            #(#lenses_struct_fields),*
        }
    );

    // Declare a `_<StructName>Lenses` instance that holds the available lenses
    // for each field in the struct, for example:
    //     const _Struct2Lenses: Struct2Lenses = Struct2Lenses {
    //         int32: Struct2Int32Lens,
    //         struct1: Struct2Struct1Lens,
    //         struct1_lenses: _Struct1Lenses
    //     };
    let lenses_const_name = format_ident!("_{}Lenses", struct_name);
    let lenses_const_fields = fields.iter().map(|field|
        // TODO: Most of this is nearly identical to how the "Lenses" struct is declared,
        // except for the underscore prefix in a couple places; might be good to consolidate
        if let Some(field_name) = &field.ident {
            let field_lens_name = format_ident!("{}{}Lens", struct_name, to_camel_case(&field_name.to_string()));
            if is_primitive(&field.ty) {
                quote!(#field_name: #field_lens_name)
            } else {
                let field_parent_lenses_field_name = format_ident!("{}_lenses", field_name);
                let field_parent_lenses_type_name = format_ident!("_{}Lenses", to_camel_case(&field_name.to_string()));
                quote!(
                    #field_name: #field_lens_name,
                    #field_parent_lenses_field_name: #field_parent_lenses_type_name
                )
            }
        } else {
            // This should be unreachable, since we already verified above that the struct
            // only contains named fields
            panic!("`#[derive(Lenses)]` may only be applied to structs with named fields")
        }
    );
    let lenses_const = quote!(
        #[allow(dead_code)]
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        #lens_visibility const #lenses_const_name: #lenses_struct_name = #lenses_struct_name {
            #(#lenses_const_fields),*
        };
    );

    // Build the output
    let expanded = quote! {
        #(#lens_items)*

        #lenses_struct

        #lenses_const
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// Return true if the given type should be considered a primitive, i.e., whether
/// it doesn't have lenses defined for it.
fn is_primitive(ty: &syn::Type) -> bool {
    let type_str = quote!(#ty).to_string();
    match type_str.as_ref() {
        // XXX: This is quick and dirty; we need a more reliable way to
        // know whether the field is a struct type for which there are
        // lenses derived
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "String" => {
            true
        }
        _ => false,
    }
}

// XXX: Lifted from librustc_lint/builtin.rs
fn to_camel_case(s: &str) -> String {
    s.split('_')
        .flat_map(|word| {
            word.chars().enumerate().map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect()
                }
            })
        })
        .collect::<Vec<_>>()
        .concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_camel_case_should_work() {
        assert_eq!(to_camel_case("this_is_snake_case"), "ThisIsSnakeCase");
    }
}
