//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use std::marker::PhantomData;

use path::LensPath;

/// A lens offers a purely functional means to access and/or modify a field that is
/// nested in an immutable data structure.
pub trait Lens {
    /// The lens source type, i.e., the object containing the field.
    type Source;

    /// The lens target type, i.e., the field to be accessed or modified.
    type Target;

    /// Returns a `LensPath` that describes the target of this lens relative to its source.
    fn path(&self) -> LensPath;
    
    /// Sets the target of the lens. (This requires a mutable source reference, and as such is typically
    /// only used internally.)
    #[doc(hidden)]
    fn mutate<'a>(&self, source: &'a mut Self::Source, target: Self::Target);

    /// Sets the target of the lens and returns the new state of the source. (This consumes the source.)
    fn set(&self, source: Self::Source, target: Self::Target) -> Self::Source {
        let mut mutable_source = source;
        {
            self.mutate(&mut mutable_source, target);
        }
        mutable_source
    }
}

/// A lens that allows the target to be accessed and mutated by reference.
pub trait RefLens: Lens {
    /// Gets a reference to the target of the lens. (This does not consume the source.)
    fn get_ref<'a>(&self, source: &'a Self::Source) -> &'a Self::Target;

    /// Gets a mutable reference to the target of the lens. (This requires a mutable source reference,
    /// and as such is typically only used internally.)
    #[doc(hidden)]
    fn get_mut_ref<'a>(&self, source: &'a mut Self::Source) -> &'a mut Self::Target;
}

/// A lens that allows the target to be accessed only by cloning or copying the target value.
pub trait ValueLens: Lens {
    /// Gets a copy of the lens target. (This does not consume the source.)
    fn get(&self, source: &Self::Source) -> Self::Target;
}

/// Modifies the target of the lens by applying a function to the current value.
/// (This currently lives outside the `Lens` trait to allow lenses to be object-safe.)
pub fn mutate_with_fn<'a, L: RefLens, F>(lens: &L, source: &'a mut L::Source, f: F)
    where F: Fn(&L::Target) -> L::Target
{
    let target = f(lens.get_ref(source));
    lens.mutate(source, target);
}

/// Modifies the target of the lens by applying a function to the current value.  This consumes the source.
/// (This currently lives outside the `Lens` trait to allow lenses to be object-safe.)
pub fn modify<L: RefLens, F>(lens: &L, source: L::Source, f: F) -> L::Source
    where F: Fn(&L::Target) -> L::Target
{
    let mut mutable_source = source;
    {
        mutate_with_fn(lens, &mut mutable_source, f);
    }
    mutable_source
}

// Automatically provides implementation of `Lens` trait for all `Box<Lens>`.
impl<L: Lens + ?Sized> Lens for Box<L> {
    type Source = L::Source;
    type Target = L::Target;

    #[inline(always)]
    fn path(&self) -> LensPath {
        (**self).path()
    }

    #[inline(always)]
    fn mutate<'a>(&self, source: &'a mut L::Source, target: L::Target) {
        (**self).mutate(source, target)
    }
}

// Automatically provides implementation of `RefLens` trait for all `Box<RefLens>`.
impl<L: RefLens + ?Sized> RefLens for Box<L> {
    #[inline(always)]
    fn get_ref<'a>(&self, source: &'a L::Source) -> &'a L::Target {
        (**self).get_ref(source)
    }

    #[inline(always)]
    fn get_mut_ref<'a>(&self, source: &'a mut L::Source) -> &'a mut L::Target {
        (**self).get_mut_ref(source)
    }
}

// Automatically provides implementation of `ValueLens` trait for all `Box<ValueLens>`.
impl<L: ValueLens + ?Sized> ValueLens for Box<L> {
    #[inline(always)]
    fn get(&self, source: &L::Source) -> L::Target {
        (**self).get(source)
    }
}

/// Returns a `Lens` over a single element at the given `index` for a `Vec<T>`.
pub const fn vec_lens<T>(index: usize) -> VecLens<T> {
    VecLens { index: index, _marker: PhantomData::<T> }
}

#[doc(hidden)]
pub struct VecLens<T> {
    index: usize,
    _marker: PhantomData<T>
}

impl<T> Lens for VecLens<T> {
    type Source = Vec<T>;
    type Target = T;

    #[inline(always)]
    fn path(&self) -> LensPath {
        LensPath::new(self.index as u64)
    }

    #[inline(always)]
    fn mutate<'a>(&self, source: &'a mut Vec<T>, target: T) {
        source[self.index] = target;
    }
}

impl<T> RefLens for VecLens<T> {
    #[inline(always)]
    fn get_ref<'a>(&self, source: &'a Vec<T>) -> &'a T {
        source.get(self.index).unwrap()
    }

    #[inline(always)]
    fn get_mut_ref<'a>(&self, source: &'a mut Vec<T>) -> &'a mut T {
        source.get_mut(self.index).unwrap()
    }
}

/// Composes a `Lens<A, B>` with another `Lens<B, C>` to produce a new `Lens<A, C>`.
pub const fn compose<LHS, RHS>(lhs: LHS, rhs: RHS) -> ComposedLens<LHS, RHS>
    where LHS: RefLens, LHS::Target: 'static, RHS: Lens<Source=LHS::Target>
{
    ComposedLens { lhs: lhs, rhs: rhs }
}

/// Composes two `Lens`es.
///
/// In pseudocode:
/// ```
///     compose(Lens<A, B>, Lens<B, C>) -> Lens<A, C>
/// ```
#[doc(hidden)]
pub struct ComposedLens<LHS, RHS> {
    /// The left-hand side of the composition.
    lhs: LHS,

    /// The right-hand side of the composition.
    rhs: RHS
}

impl<LHS, RHS> Lens for ComposedLens<LHS, RHS>
    where LHS: RefLens, LHS::Target: 'static, RHS: Lens<Source=LHS::Target>
{
    type Source = LHS::Source;
    type Target = RHS::Target;

    #[inline(always)]
    fn path(&self) -> LensPath {
        LensPath::concat(self.lhs.path(), self.rhs.path())
    }

    #[inline(always)]
    fn mutate<'a>(&self, source: &'a mut LHS::Source, target: RHS::Target) {
        let rhs_source = self.lhs.get_mut_ref(source);
        self.rhs.mutate(rhs_source, target)
    }
}

impl<LHS, RHS> RefLens for ComposedLens<LHS, RHS>
    where LHS: RefLens, LHS::Target: 'static, RHS: RefLens<Source=LHS::Target>
{
    #[inline(always)]
    fn get_ref<'a>(&self, source: &'a LHS::Source) -> &'a RHS::Target {
        self.rhs.get_ref(self.lhs.get_ref(source))
    }

    #[inline(always)]
    fn get_mut_ref<'a>(&self, source: &'a mut LHS::Source) -> &'a mut RHS::Target {
        self.rhs.get_mut_ref(self.lhs.get_mut_ref(source))
    }
}

impl<LHS, RHS> ValueLens for ComposedLens<LHS, RHS>
    where LHS: RefLens, LHS::Target: 'static, RHS: ValueLens<Source=LHS::Target>
{
    #[inline(always)]
    fn get(&self, source: &LHS::Source) -> RHS::Target {
        self.rhs.get(self.lhs.get_ref(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use path::*;
    
    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct1 {
        foo: i32,
        bar: i16
    }

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct2 {
        baz: i32,
        inner: Struct1
    }

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct3 {
        inner: Struct2
    }

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct4 {
        inner_vec: Vec<Struct1>
    }

    #[test]
    fn a_basic_lens_should_work() {
        let lens = lens!(Struct1.foo);

        let s0 = Struct1 { foo: 42, bar: 73 };
        assert_eq!(*lens.get_ref(&s0), 42);

        let s1 = lens.set(s0, 41);
        assert_eq!(s1.foo, 41);
        assert_eq!(s1.bar, 73);

        let s2 = modify(&lens, s1, |a| a - 1);
        assert_eq!(s2.foo, 40);
        assert_eq!(s2.bar, 73);
    }

    #[test]
    fn a_vec_lens_should_work() {
        let lens = vec_lens::<u32>(1);

        let v0 = vec!(0u32, 1, 2);
        assert_eq!(*lens.get_ref(&v0), 1);

        let v1 = lens.set(v0, 42);
        assert_eq!(v1, vec!(0u32, 42, 2));

        let v2 = modify(&lens, v1, |a| a - 1);
        assert_eq!(v2, vec!(0u32, 41, 2));
    }

    #[test]
    fn the_lens_macro_should_support_vec_indexing() {
        let lens = lens!(Struct4.inner_vec[1].foo);

        let s0 = Struct4 { inner_vec: vec!(
            Struct1 { foo: 42, bar: 73 },
            Struct1 { foo: 110, bar: 210 }
        )};
        assert_eq!(*lens.get_ref(&s0), 110);

        let s1 = lens.set(s0, 111);
        assert_eq!(s1.inner_vec[1].foo, 111);

        let s2 = modify(&lens, s1, |a| a + 1);
        assert_eq!(s2.inner_vec[1].foo, 112);
    }
    
    #[test]
    fn lens_composition_should_work() {
        let lens = lens!(Struct3.inner.inner.foo);

        let s0 = Struct3 { inner: Struct2 { baz: 123, inner: Struct1 { foo: 42, bar: 73 } } };
        assert_eq!(*lens.get_ref(&s0), 42);

        let s1 = lens.set(s0, 41);
        assert_eq!(s1.inner.baz, 123);
        assert_eq!(s1.inner.inner.foo, 41);
        assert_eq!(s1.inner.inner.bar, 73);

        let s2 = modify(&lens, s1, |a| a - 1);
        assert_eq!(s2.inner.baz, 123);
        assert_eq!(s2.inner.inner.foo, 40);
        assert_eq!(s2.inner.inner.bar, 73);
    }

    #[test]
    fn lens_composition_should_work_with_boxed_lenses() {
        let foo_lens: Box<RefLens<Source=Struct1, Target=i32>> = Box::new(Struct1FooLens);
        let lens = compose_lens!(Struct3InnerLens, Box::new(Struct2InnerLens), foo_lens);

        let s0 = Struct3 { inner: Struct2 { baz: 123, inner: Struct1 { foo: 42, bar: 73 } } };
        assert_eq!(*lens.get_ref(&s0), 42);

        let s1 = lens.set(s0, 41);
        assert_eq!(s1.inner.baz, 123);
        assert_eq!(s1.inner.inner.foo, 41);
        assert_eq!(s1.inner.inner.bar, 73);

        let s2 = modify(&lens, s1, |a| a - 1);
        assert_eq!(s2.inner.baz, 123);
        assert_eq!(s2.inner.inner.foo, 40);
        assert_eq!(s2.inner.inner.bar, 73);
    }
}
