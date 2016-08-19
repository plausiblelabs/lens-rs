//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use std::ops::Not;

use num::{PrimInt, One};

use lens::*;
use transform::*;

/// A transform that invokes a lens `set` operation with the output of the given function.
pub fn set_tx<L, F>(lens: L, func: F) -> impl Transform<Input=L::Source, Output=L::Source>
    where L: Lens, F: Fn(&L::Source) -> L::Target
{
    LensSetTransform { lens: lens, func: func }
}

/// A transform that invokes a lens `modify` operation with the given function.
pub fn mod_tx<L, F>(lens: L, func: F) -> impl Transform<Input=L::Source, Output=L::Source>
    where L: RefLens, F: Fn(&L::Target) -> L::Target
{
    LensModifyTransform { lens: lens, func: func }
}

/// A transform that increments the integral target value for the given lens.
pub fn increment_tx<L>(lens: L) -> impl Transform<Input=L::Source, Output=L::Source>
    where L: RefLens, L::Target: PrimInt
{
    // TODO: Ideally we would piggyback on `mod_tx` for this, but that would probably require removing `F` from
    // the type signature one way or another
    LensIncrementTransform { lens: lens }
}

/// A transform that decrements the integral target value for the given lens.
pub fn decrement_tx<L>(lens: L) -> impl Transform<Input=L::Source, Output=L::Source>
    where L: RefLens, L::Target: PrimInt
{
    LensDecrementTransform { lens: lens }
}

/// A transform that negates the boolean target value for the underlying lens.
pub fn not_tx<T, L>(lens: L) -> impl Transform<Input=L::Source, Output=L::Source>
    where L: RefLens<Target=T>, T: Not<Output=T> + Clone
{
    LensNotTransform { lens: lens }
}

/// A transform that invokes a lens `set` operation with the output of the given function.
struct LensSetTransform<L, F> {
    /// The underlying lens.
    lens: L,

    /// A closure that takes the lens source by reference and produces a new lens target value.
    func: F
}

impl<L, F> Transform for LensSetTransform<L, F>
    where L: Lens, F: Fn(&L::Source) -> L::Target
{
    type Input = L::Source;
    type Output = L::Source;
    
    fn apply(&self, input: L::Source) -> L::Source {
        let new_value = (&self.func)(&input);
        self.lens.set(input, new_value)
    }
}

/// A transform that invokes a lens `modify` operation with the given function.
struct LensModifyTransform<L, F> {
    /// The underlying lens.
    lens: L,

    /// A closure that takes the lens target value and produces a new target value.
    func: F
}

impl<L, F> Transform for LensModifyTransform<L, F>
    where L: RefLens, F: Fn(&L::Target) -> L::Target
{
    type Input = L::Source;
    type Output = L::Source;
    
    fn apply(&self, input: L::Source) -> L::Source {
        let target = (self.func)(self.lens.get_ref(&input));
        self.lens.set(input, target)
    }
}

/// A transform that increments the integral target value of the underlying lens.
struct LensIncrementTransform<L> {
    /// The underlying lens.
    lens: L
}

impl<L> Transform for LensIncrementTransform<L>
    where L: RefLens, L::Target: PrimInt
{
    type Input = L::Source;
    type Output = L::Source;
    
    fn apply(&self, input: L::Source) -> L::Source {
        let target = *self.lens.get_ref(&input) + L::Target::one();
        self.lens.set(input, target)
    }
}

/// A transform that decrements the integral target value of the underlying lens.
struct LensDecrementTransform<L> {
    /// The underlying lens.
    lens: L
}

impl<L> Transform for LensDecrementTransform<L>
    where L: RefLens, L::Target: PrimInt
{
    type Input = L::Source;
    type Output = L::Source;
    
    fn apply(&self, input: L::Source) -> L::Source {
        let target = *self.lens.get_ref(&input) - L::Target::one();
        self.lens.set(input, target)
    }
}

/// A transform that applies a unary `!` to the boolean target value of the underlying lens.
struct LensNotTransform<L> {
    /// The underlying lens.
    lens: L
}

impl<T, L> Transform for LensNotTransform<L>
    where L: RefLens<Target=T>, T: Not<Output=T> + Clone
{
    type Input = L::Source;
    type Output = L::Source;
    
    fn apply(&self, input: L::Source) -> L::Source {
        let target = self.lens.get_ref(&input).clone();
        self.lens.set(input, !target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lens::*;
    use path::*;
    use transform::*;

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct1 {
        foo: i32,
        bar: i16
    }

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct2 {
        enabled: bool
    }

    #[test]
    fn a_lens_set_transform_should_work() {
        let tx = set_tx(lens!(Struct1.foo), |s| s.foo + 42);
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 42);
    }

    #[test]
    fn a_lens_modify_transform_should_work() {
        let tx = mod_tx(lens!(Struct1.foo), |foo| foo + 42);
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 42);
    }

    #[test]
    fn a_lens_increment_transform_should_work() {
        let tx = increment_tx(lens!(Struct1.foo));
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 1);
    }

    #[test]
    fn a_lens_decrement_transform_should_work() {
        let tx = decrement_tx(lens!(Struct1.foo));
        let s0 = Struct1 { foo: 42, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 41);
    }

    #[test]
    fn a_lens_not_transform_should_work() {
        let tx = not_tx(lens!(Struct2.enabled));
        let s0 = Struct2 { enabled: false };
        let s1 = tx.apply(s0);
        assert_eq!(s1.enabled, true);
    }

    #[test]
    fn lens_transform_composition_should_work() {
        let add_one = mod_tx(lens!(Struct1.foo), |foo| foo + 1);
        let add_two = mod_tx(lens!(Struct1.foo), |foo| foo + 2);
        let mul_two = mod_tx(lens!(Struct1.foo), |foo| foo * 2);
        let tx = compose_tx!(add_one, add_two, mul_two);
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 6);
    }
}
