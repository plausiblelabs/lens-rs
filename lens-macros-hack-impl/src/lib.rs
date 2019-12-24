//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_hack::proc_macro_hack;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, ExprField, Member};

#[proc_macro_hack]
pub fn lens(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let expr = parse_macro_input!(input as Expr);

    // Check that the expression is a "named struct field access"
    let lens_parts: Vec<String>;
    if let Expr::Field(field_access) = &expr {
        // Extract the list of lens names
        match extract_lens_parts(&field_access) {
            Ok(parts) => {
                lens_parts = parts;
            }
            Err(error) => {
                return error.to_compile_error().into();
            }
        }
    } else {
        return syn::Error::new(expr.span(), "lens!() expression must be structured like a field access, e.g. `Struct.outer_field.inner_field`").to_compile_error().into();
    }

    // At this point we should have at least two parts: the root struct name
    // and the first field name
    if lens_parts.len() < 2 {
        return syn::Error::new(expr.span(), "lens!() expression must be structured like a field access, e.g. `Struct.outer_field.inner_field`").to_compile_error().into();
    }

    // We can build up the composed lens by concatenating the parts of the
    // expression (inserting `Lenses` or `_lenses` as needed); this relies
    // on the fact that the `#derive(Lenses)` macro creates a special
    // `struct FooLenses` for each source struct that enumerates the
    // lens type name for each field.
    //
    // For example, suppose we have the following lens expression:
    //     lens!(Struct3.struct2.struct1.int32)
    //
    // We extracted the parts into `lens_parts` above, producing:
    //     ["Struct3", "struct2", "struct1", "int32"]
    //
    // Now we can access the lenses and compose them together:
    //     compose_lens!(
    //         _Struct3Lenses.struct2,
    //         _Struct3Lenses.struct2_lenses.struct1,
    //         _Struct3Lenses.struct2_lenses.struct1_lenses.int32
    //     )
    let mut parent_lenses_name = format_ident!("_{}Lenses", lens_parts[0]);
    let mut child_field_name = format_ident!("{}", lens_parts[1]);
    let mut base_lens_expr = quote!(#parent_lenses_name);
    let mut lens_expr = quote!(#base_lens_expr.#child_field_name);
    let mut lens_exprs: Vec<TokenStream2> = vec![lens_expr.clone()];

    for i in 2..lens_parts.len() {
        let prev_base_lens_expr = base_lens_expr.clone();
        let prev_child_field_name = child_field_name.clone();
        parent_lenses_name = format_ident!("{}_lenses", prev_child_field_name);
        child_field_name = format_ident!("{}", lens_parts[i]);
        base_lens_expr = quote!(#prev_base_lens_expr.#parent_lenses_name);
        lens_expr = quote!(#base_lens_expr.#child_field_name);
        lens_exprs.push(lens_expr.clone());
    }

    // Build the output
    let expanded = quote! {
        compose_lens!(#(#lens_exprs),*);
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

/// Given an expression like `Struct1.struct2_field.struct3_field`, recurse until we hit the root
/// struct and then build a list of lens names that can be passed to `compose_lens!`.
/// For example, the above expression would result in the following list of identifiers:
/// ```
///    [Struct1, struct2_field, struct3_field]
/// ```
fn extract_lens_parts(field_access: &ExprField) -> Result<Vec<String>, syn::Error> {
    // Look at the parent to determine if we're at the root, or if this is a chained field access
    let base_parts = match &*field_access.base {
        Expr::Path(base_expr_path) => {
            // We hit the root of the expression; extract the struct name
            let path_segments = &base_expr_path.path.segments;
            if path_segments.len() > 1 {
                Err(syn::Error::new(field_access.span(), "lens!() expression must start with unqualified struct name, e.g. `Struct.outer_field.inner_field`"))
            } else {
                let struct_name = path_segments[0].ident.to_string();
                Ok(vec![struct_name])
            }
        }
        Expr::Field(base_field_access) => {
            // This is another field access; extract the base portion first
            extract_lens_parts(&base_field_access)
        }
        _ => {
            Err(syn::Error::new(field_access.span(), "lens!() expression must be structured like a field access, e.g. `Struct.outer_field.inner_field`"))
        }
    };

    // Append the field name
    base_parts.and_then(|parts| {
        if let Member::Named(field_ident) = &field_access.member {
            let mut new_parts = parts.clone();
            new_parts.push(field_ident.to_string());
            Ok(new_parts)
        } else {
            Err(syn::Error::new(
                field_access.span(),
                "lens!() only works with named fields, e.g. `Struct.outer_field.inner_field`",
            ))
        }
    })
}
