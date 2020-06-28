//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use proc_macro_hack::proc_macro_hack;

// Re-export the pl-lens-derive crate
pub use pl_lens_derive::*;

/// This is a macro-based shorthand that allows us to write:
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
#[proc_macro_hack]
pub use pl_lens_macros::lens;

// The following is necessary to make exported macros visible.
#[macro_use]
mod macros;
pub use self::macros::*;

mod lens;
mod path;

pub use self::lens::*;
pub use self::path::*;
