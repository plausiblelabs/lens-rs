//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

// The following allows for constant functions.
#![feature(const_fn)]

// The following allows for macro debugging via trace_macros(true/false).
#![feature(trace_macros)]

// The following allows for using macros defined in the separate lens_macros crate.
#![feature(plugin, custom_attribute)]
#![plugin(lens_macros)]

// The following is necessary to make exported macros visible.
#[macro_use]
mod macros;

mod transform;
mod lens;

pub use self::macros::*;
pub use self::lens::*;
pub use self::transform::*;

extern crate num;
