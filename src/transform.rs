//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use std::marker::PhantomData;

/// Transforms an input and produces an output.
pub trait Transform {
    /// The input type.
    type Input;

    /// The output type.
    type Output;

    /// Transforms the input value and returns an output value.
    fn apply(&self, input: Self::Input) -> Self::Output;
}

/// Composes a `Transform<A, B>` with another `Transform<B, C>` to produce a new `Transform<A, C>`.
pub const fn composed_tx<LHS, RHS>(lhs: LHS, rhs: RHS) -> impl Transform<Input=LHS::Input, Output=RHS::Output>
    where LHS: Transform, RHS: Transform<Input=LHS::Output>
{
    ComposedTransform { lhs: lhs, rhs: rhs }
}

/// Composes two `Transform`s.
///
/// In pseudocode:
/// ```
///     compose(Transform<A, B>, Transform<B, C>) -> Transform<A, C>
/// ```
struct ComposedTransform<LHS, RHS> {
    /// The left-hand side of the composition.
    lhs: LHS,

    /// The right-hand side of the composition.
    rhs: RHS
}

impl<LHS, RHS> Transform for ComposedTransform<LHS, RHS>
    where LHS: Transform, RHS: Transform<Input=LHS::Output>
{
    type Input = LHS::Input;
    type Output = RHS::Output;
    
    fn apply(&self, input: LHS::Input) -> RHS::Output {
        self.rhs.apply(self.lhs.apply(input))
    }
}

/// The identity transform that simply passes the input data through as the output.
pub fn identity_tx<X>() -> impl Transform<Input=X, Output=X> {
    IdentityTransform { _marker: PhantomData::<X> }
}

struct IdentityTransform<X> {
    _marker: PhantomData<X>
}

impl<X> Transform for IdentityTransform<X> {
    type Input = X;
    type Output = X;
    
    fn apply(&self, input: X) -> X {
        input
    }
}

/// A transform that applies a given function to the input.
pub fn fn_tx<X, Y, F>(f: F) -> impl Transform<Input=X, Output=Y>
    where F: Fn(X) -> Y
{
    FnTransform { func: f, _x_marker: PhantomData::<X>, _y_marker: PhantomData::<Y> }
}

struct FnTransform<X, Y, F> {
    /// A closure that takes the transform input value and produces an output value.
    func: F,
    _x_marker: PhantomData<X>,
    _y_marker: PhantomData<Y>
}

impl<X, Y, F> Transform for FnTransform<X, Y, F>
    where F: Fn(X) -> Y
{
    type Input = X;
    type Output = Y;
    
    fn apply(&self, input: X) -> Y {
        (self.func)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identity_transform_should_pass_input_through_unchanged() {
        let tx = identity_tx::<u8>();
        assert_eq!(tx.apply(42u8), 42u8);
    }

    #[test]
    fn a_fn_transform_should_work() {
        let tx = fn_tx(|x: u32| x * 2);
        let x = 21u32;
        let y = tx.apply(x);
        assert_eq!(y, 42);
    }

    #[test]
    fn transform_composition_should_work() {
        let add_one = fn_tx(|x: u32| x + 1);
        let add_two = fn_tx(|x: u32| x + 2);
        let mul_two = fn_tx(|x: u32| x * 2);
        let tx = composed_tx(add_one, composed_tx(add_two, mul_two));
        let x = 0u32;
        let y = tx.apply(x);
        assert_eq!(y, 6);
    }
}
