//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
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

/// Provides a shorthand for composing a series of transforms.
#[macro_export]
macro_rules! compose_tx {
    { $head:expr } => {
        $head
    };
    { $head:expr, $($tail:expr),+ } => {
        composed_tx($head, compose_tx!($($tail),+))
    };
}

/// Provides a shorthand for transforming a struct through a series of lens `set` operations.
#[macro_export]
macro_rules! lens_set {
    { $in_state:expr, $sname:ident, $({$($st:tt)+}),+ } => {
        {
            use lens::Transform;
            compose_tx!(
                $(split_lens_set_stmt!([$sname] $($st)+)),+
            ).apply($in_state)
        }
    };
}

/// Splits a statement of the form `x <- y` around the arrow.
#[macro_export]
#[doc(hidden)]
macro_rules! split_lens_set_stmt {
    { [$sname:ident $($stack:tt)*] $x:tt <- $($rest:tt)+ } => {
        lens_set_stmt!($sname, [$($stack)* $x], [$($rest)+])
    };
    { [$sname:ident $($stack:tt)*] $x:tt $($rest:tt)* } => {
        split_lens_set_stmt!([$sname $($stack)* $x] $($rest)*)
    };
}

/// Processes a single lens `set` statement, either taking a closure or an expression.
#[macro_export]
#[doc(hidden)]
macro_rules! lens_set_stmt {
    { $sname:ident, [$($lhs:tt)+], [|$arg:ident| $e:expr] } => {
        lens_set_with_fn!($sname, [$($lhs)+], $arg, $e)
    };
    { $sname:ident, [$($lhs:tt)+], [$e:expr] } => {
        lens_set_with_expr!($sname, [$($lhs)+], $e)
    };
}

/// Processes a single lens `set` statement that takes an expression.
#[macro_export]
#[doc(hidden)]
macro_rules! lens_set_with_expr {
    { $sname:ident, [$($lhs:tt)+], $e:expr } => {
        lens_set_left_side!([$sname] $($lhs)+).set_tx(|_| $e)
    };
}

/// Processes a single lens `set` statement that takes a closure.
#[macro_export]
#[doc(hidden)]
macro_rules! lens_set_with_fn {
    { $sname:ident, [$($lhs:tt)+], $arg:ident, $e:expr } => {
        {
            lens_set_left_side!([$sname] $($lhs)+).set_tx(|$arg| $e)
        }
    };
}

/// Processes the left-hand side of a lens `set` statement by expanding each L(x.y.z) to a lens.
#[macro_export]
#[doc(hidden)]
macro_rules! lens_set_left_side {
    { [$sname:ident $($stack:tt)*] } => {
        lens_expr!($($stack)*)
    };
    { [$sname:ident $($stack:tt)*] L($($lens:tt)+) $($rest:tt)* } => {
        lens_set_left_side!([$sname $($stack)* lens!($sname.$($lens)+)] $($rest)*)
    };
    { [$sname:ident $($stack:tt)*] $x:tt $($rest:tt)* } => {
        lens_set_left_side!([$sname $($stack)* $x] $($rest)*)
    };
}

/// Massages a stack of token trees produced by the `lens_set_stmt` macro so that they evaluate to an expression.
#[macro_export]
#[doc(hidden)]
macro_rules! lens_expr {
    { $stack:expr } => {
        $stack
    };
}
