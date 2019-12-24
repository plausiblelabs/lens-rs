//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use proc_macro_hack::proc_macro_hack;

/// This is a fairly ridiculous implementation of a lens! shorthand that allows us to write:
///
/// ```text,no_run
///   lens!(SomeStruct.foo.bar_vec[3].baz)
/// ```
///
/// instead of:
///
/// ```text,no_run
///   compose_lens!(SomeStructFooLens, FooBarVecLens, vec_lens::<BarThing>(3), BarThingBazLens)
/// ```
///
/// It relies on our lens_impl! macro to generate a nested macro that can resolve to the target type
/// of the lens.  We then eagerly invoke that macro while we're parsing the lens! arguments.  All of
/// this is to make up for the fact that we don't have a way to inspect type information for arbitrary
/// types at compile/parse time.  This is all probably very fragile; a more robust implementation
/// would account for complex types and use fully-qualified identifiers, etc.
#[proc_macro_hack]
pub use pl_lens_macros_hack_impl::lens;
