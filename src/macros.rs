//
// Copyright (c) 2015-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

/// Provides a shorthand for composing a series of lenses.
#[macro_export]
macro_rules! compose_lens {
    { $head:expr } => {
        $head
    };
    { $head:expr, $($tail:expr),+ } => {
        compose($head, compose_lens!($($tail),+))
    };
}
