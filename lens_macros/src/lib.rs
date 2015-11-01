//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

#![crate_type = "dylib"]
#![feature(rustc_private, plugin_registrar, quote, slice_patterns, vec_push_all, convert)]

extern crate syntax;
extern crate rustc;

use syntax::ast;
use syntax::ast::{Ident, Item, ItemStruct, MetaItem, StructFieldKind, TokenTree, TtToken, TtDelimited, VariantData};
use syntax::attr;
use syntax::attr::AttrMetaMethods;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::parse::token::intern;
use syntax::ext::base::{Annotatable, ExtCtxt, MacResult, MacEager, MultiItemDecorator, DummyResult, SyntaxExtension, expr_to_string};
use syntax::print::pprust::ty_to_string;
use syntax::ptr::P;
use rustc::plugin::Registry;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("Lensed"), SyntaxExtension::MultiDecorator(Box::new(LensedDecorator)));
    reg.register_macro("lens", expand_lens);
}

/// Handles the `Lensed` attribute applied to a struct by generating a `Lens` implementation for
/// each field in the struct.
struct LensedDecorator;
impl MultiItemDecorator for LensedDecorator {
    fn expand(&self, cx: &mut ExtCtxt, _span: Span, mitem: &MetaItem, item: &Annotatable, push: &mut FnMut(Annotatable)) {
        match *item {
            Annotatable::Item(ref struct_item) => {
                match struct_item.node {
                    ItemStruct(VariantData::Struct(ref struct_fields, _), _) => {
                        for (index, spanned_field) in struct_fields.iter().enumerate() {
                            let field = &spanned_field.node;
                            match field.kind {
                                StructFieldKind::NamedField(ref ident, _) => {
                                    let no_lens = field.attrs.iter().any(|attr| {
                                        match attr.node.value.node {
                                            ast::MetaWord(ref name) if name == &"NoLens" => {
                                                attr::mark_used(&attr);
                                                true
                                            }
                                            _ => {
                                                false
                                            }
                                        }
                                    });
                                    if !no_lens {
                                        derive_lens(cx, push, &struct_item, ident, &field.ty, index as u64);
                                    }
                                }
                                _ => {
                                    cx.span_err(mitem.span, "`Lensed` may only be applied to structs with named fields");
                                    return;
                                }
                            }
                        }
                    }

                    _ => {
                        cx.span_err(mitem.span, "`Lensed` may only be applied to structs");
                        return;
                    }
                }
            }

            _ => {
                cx.span_err(mitem.span, "`Lensed` may only be applied to struct items");
                return;
            }
        }
    }
}

/// Generates a `Lens` implementation for a struct field.
#[allow(unused_imports, unused_mut)]
fn derive_lens(cx: &mut ExtCtxt, push: &mut FnMut(Annotatable), existing_item: &Item,
               field_name: &Ident, field_type: &P<ast::Ty>, field_index: u64)
{
    // Extract the struct ident
    let struct_ident = existing_item.ident;
    
    // Build the Lens name from the struct name and field name
    let lens_name = build_ident(format!("{}{}Lens", struct_ident.name, to_camel_case(&field_name.name.as_str())));

    // Build the name of the macro that expands to the lens target type
    let lens_type_macro_name = build_ident(format!("{}_{}_lens_target_type", struct_ident.name.as_str().to_lowercase(), field_name.name.as_str().to_lowercase()));

    // Push the lens struct declaration item
    let lens_visibility = match existing_item.vis {
        ast::Visibility::Public => quote_tokens!(cx, pub),
        ast::Visibility::Inherited => quote_tokens!(cx, )
    };
    push_new_item(push, existing_item, quote_item!(
        cx,
        #[allow(dead_code)]
        #[doc(hidden)]
        $lens_visibility struct $lens_name;
    ));

    // Push the `Lens` impl item
    push_new_item(push, existing_item, quote_item!(
        cx,
        #[allow(dead_code)]
        impl Lens for $lens_name {
            type Source = $struct_ident;
            type Target = $field_type;

            #[inline(always)]
            fn path(&self) -> LensPath {
                LensPath::new($field_index)
            }

            #[inline(always)]
            fn mutate<'a>(&self, source: &'a mut $struct_ident, target: $field_type) {
                source.$field_name = target
            }
        }
    ));

    // Push the `RefLens` impl item
    push_new_item(push, existing_item, quote_item!(
        cx,
        #[allow(dead_code)]
        impl RefLens for $lens_name {
            #[inline(always)]
            fn get_ref<'a>(&self, source: &'a $struct_ident) -> &'a $field_type {
                &(*source).$field_name
            }

            #[inline(always)]
            fn get_mut_ref<'a>(&self, source: &'a mut $struct_ident) -> &'a mut $field_type {
                &mut (*source).$field_name
            }
        }
    ));

    // Push a `ValueLens` impl item if the target is a primitive
    // TODO: Should do this automatically for any target type that implements `Clone`
    let value_lens = {
        let field_type_str = ty_to_string(field_type);
        match field_type_str.as_ref() {
            "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => true,
            _ => false
        }
    };
    if value_lens {
        push_new_item(push, existing_item, quote_item!(
            cx,
            #[allow(dead_code)]
            impl ValueLens for $lens_name {
                #[inline(always)]
                fn get(&self, source: &$struct_ident) -> $field_type {
                    (*source).$field_name.clone()
                }
            }
        ));
    }

    // Push the lens type macro item
    push_new_item(push, existing_item, quote_item!(
        cx,
        #[doc(hidden)]
        #[macro_export]
        macro_rules! $lens_type_macro_name {
            {} => {
                stringify!($field_type)
            }
        }
    ));
}

/// Pushes the given item into the AST.  The new item will inherit the lint attributes of the existing item
/// from which the new item was derived.
fn push_new_item(push: &mut FnMut(Annotatable), existing_item: &Item, new_item_ptr: Option<P<Item>>) {
    let new_item = new_item_ptr.unwrap();
    
    // Keep the lint attributes of the previous item to control how the
    // generated implementations are linted
    let mut attrs = new_item.attrs.clone();
    attrs.extend(existing_item.attrs.iter().filter(|a| {
        match &a.name()[..] {
            "allow" | "warn" | "deny" | "forbid" => true,
            _ => false,
        }
    }).cloned());

    // Push the new item into the AST
    push(Annotatable::Item(P(ast::Item {
        attrs: attrs,
        ..(*new_item).clone()
    })))
}

/// This is a fairly ridiculous implementation of a lens! shorthand that allows us to write:
///
/// ```
///   lens!(SomeStruct.foo.bar_vec[3].baz)
/// ```
///
/// instead of:
///
/// ```
///   compose_lens!(SomeStructFooLens, FooBarVecLens, vec_lens::<BarThing>(3), BarThingBazLens)
/// ```
///
/// It relies on our lens_impl! macro to generate a nested macro that can resolve to the target type
/// of the lens.  We then eagerly invoke that macro while we're parsing the lens! arguments.  All of
/// this is to make up for the fact that we don't have a way to inspect type information for arbitrary
/// types at compile/parse time.  This is all probably very fragile; a more robust implementation
/// would account for complex types and use fully-qualified identifiers, etc.
fn expand_lens(cx: &mut ExtCtxt, span: Span, args: &[TokenTree]) -> Box<MacResult> {
    let usage_error = "lens! macro expects arguments in the form: struct_name.field_name || struct_name.vec_field_name[index]";
    
    // Args should look something like: [Ident, Dot, Ident, Dot, Ident]
    if args.len() < 3 {
        cx.span_err(span, usage_error);
        return DummyResult::any(span);
    }

    // Extract the initial struct ident
    let mut struct_ident = match args[0] {
        TtToken(_, token::Ident(ident, _)) => ident,
        _ => {
            cx.span_err(span, usage_error);
            return DummyResult::any(span);
        }
    };

    // Extract the field tokens
    let field_tokens = &args[1..];

    // Extract each field name and resolve the lens type
    let mut token_index = 0;
    let mut lens_args: Vec<TokenTree> = Vec::new();
    loop {
        // Determine whether this is a lens for a struct field (struct.field) or a vec element (vec[index])
        match field_tokens[token_index] {
            TtToken(_, token::Dot) => {
                // This is (hopefully) a struct field reference
                token_index += 1;

                // Extract the field name
                let field_name = match field_tokens[token_index] {
                    TtToken(_, token::Ident(ident, _)) => ident.name,
                    _ => {
                        cx.span_err(span, usage_error);
                        return DummyResult::any(span);
                    }
                };
                        
                // Build the lens name for this (source, target) pair
                let lens_name = format!("{}{}Lens", struct_ident.name, to_camel_case(&field_name.as_str()));

                // Add the lens identifier to the list of args
                let lens_ident = token::str_to_ident(&lens_name);
                lens_args.push(TtToken(span, token::Ident(lens_ident, token::IdentStyle::Plain)));

                // Stop when there are no more fields to parse
                token_index += 1;
                if token_index == field_tokens.len() {
                    break;
                }

                // We need to manually comma-separate the args that we'll pass to the compose_lens! macro
                lens_args.push(TtToken(span, token::Comma));
                
                // Resolve the type of the target, which will be used to construct the next lens name
                // XXX: This is super evil!  We assume that there's a macro alongside each lens, and
                // that it is named according to a certain convention.  We use `expr_to_string` to
                // evaluate that macro eagerly in order to resolve the target type of the lens.
                let macro_name = format!("{}_{}_lens_target_type", struct_ident.name.as_str().to_lowercase(), field_name);
                let macro_ident = token::str_to_ident(&macro_name);
                let target_type_expr = quote_expr!(cx, $macro_ident!());
                let target_type = match expr_to_string(cx, target_type_expr, &format!("Failed to resolve lens target type for struct {} and field {}", struct_ident, field_name)) {
                    Some((s, _)) => {
                        s
                    },
                    None => {
                        return DummyResult::any(span);
                    }
                };
        
                struct_ident = token::str_to_ident(&target_type);
            }
            
            TtDelimited(_, ref delimited) => {
                // This is (hopefully) an index for a vec lens
                // TODO: This pattern matching code is awful; needs cleanup
                match **delimited {
                    ast::Delimited { delim: token::Bracket, tts: ref delimited_tts, .. } => {
                        match delimited_tts.as_slice() {
                            [ref vec_index @ TtToken(_, token::Literal(token::Integer(_), _))] => {
                                // Extract the Vec element type
                                // TODO: This is very fragile; need a safer way to identify the element type
                                let vec_str = struct_ident.name.as_str().replace(" ", "");
                                if !vec_str.starts_with("Vec<") || !vec_str.ends_with(">") {
                                    cx.span_err(span, "lens! macro only supports indexing where source is `Vec<T>`");
                                    return DummyResult::any(span);
                                }
                                let vec_element_type = token::str_to_ident(&vec_str[4..vec_str.len() - 1]);
                                
                                // Add the vec_lens function call to the list of args
                                lens_args.push_all(&quote_tokens!(cx, vec_lens::<$vec_element_type>($vec_index)));

                                // Stop when there are no more fields to parse
                                token_index += 1;
                                if token_index == field_tokens.len() {
                                    break;
                                }

                                // We need to manually comma-separate the args that we'll pass to the compose_lens! macro
                                lens_args.push(TtToken(span, token::Comma));

                                // Use the Vec element type as the source type for the next lens, if any
                                struct_ident = vec_element_type;
                            }
                            _ => {
                                cx.span_err(span, usage_error);
                                return DummyResult::any(span);
                            }
                        }
                    }
                    _ => {
                        cx.span_err(span, usage_error);
                        return DummyResult::any(span);
                    }
                }
            }
            
            _ => {
                cx.span_err(span, usage_error);
                return DummyResult::any(span);
            }
        }
    }

    MacEager::expr(
        quote_expr!(cx, compose_lens!($lens_args);)
    )
}

/// Builds an identifier from the given string.
fn build_ident(id_str: String) -> P<ast::Ident> {
    P(token::str_to_ident(&id_str[..]))
}

// XXX: Lifted from librustc_lint/builtin.rs
fn to_camel_case(s: &str) -> String {
    s.split('_').flat_map(|word| word.chars().enumerate().map(|(i, c)|
        if i == 0 {
            c.to_uppercase().collect::<String>()
        } else {
            c.to_lowercase().collect()
        }
    )).collect::<Vec<_>>().concat()
}
