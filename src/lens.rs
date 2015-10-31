//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use std::fmt;
use std::marker::PhantomData;
use std::ops::Not;

use num::{PrimInt, One};

use transform::Transform;

/// An element in a `LensPath`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct LensPathElement {
    id: u64
}

impl LensPathElement {
    pub fn new(id: u64) -> LensPathElement {
        LensPathElement { id: id }
    }
}

/// Describes a lens relative to a source data structure.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct LensPath {
    /// The path elements.
    pub elements: Vec<LensPathElement>
}

impl LensPath {
    /// Creates a new `LensPath` with no elements.
    pub fn empty() -> LensPath {
        LensPath {
            elements: vec![]
        }
    }
    
    /// Creates a new `LensPath` with a single element.
    pub fn new(id: u64) -> LensPath {
        LensPath {
            elements: vec![LensPathElement { id: id }]
        }
    }

    /// Creates a new `LensPath` with a single index (for an indexed type such as `Vec`).
    pub fn from_index(index: usize) -> LensPath {
        LensPath {
            elements: vec![LensPathElement { id: index as u64 }]
        }
    }

    /// Creates a new `LensPath` with two elements.
    pub fn from_pair(id0: u64, id1: u64) -> LensPath {
        LensPath {
            elements: vec![LensPathElement { id: id0 }, LensPathElement { id: id1 }]
        }
    }

    /// Creates a new `LensPath` from a vector of element identifiers.
    pub fn from_vec(ids: Vec<u64>) -> LensPath {
        LensPath {
            elements: ids.iter().map(|id| LensPathElement { id: *id }).collect()
        }
    }

    /// Creates a new `LensPath` that is the concatenation of the two paths.
    pub fn concat(lhs: LensPath, rhs: LensPath) -> LensPath {
        let mut elements = lhs.elements;
        elements.extend(&rhs.elements);
        LensPath {
            elements: elements
        }
    }
}

impl fmt::Debug for LensPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.elements.iter().map(|elem| elem.id.to_string()).collect::<Vec<String>>().join(", "))
    }
}

#[test]
fn test_lens_path_concat() {
    let p0 = LensPath::from_vec(vec![1, 2, 3]);
    let p1 = LensPath::from_vec(vec![4, 5]);
    let p2 = LensPath::concat(p0, p1);
    assert_eq!(p2, LensPath::from_vec(vec![1, 2, 3, 4, 5]));
}

#[test]
fn test_lens_path_debug() {
    let path = LensPath::from_vec(vec![1, 2, 3, 4, 5]);
    assert_eq!(format!("{:?}", path), "[1, 2, 3, 4, 5]".to_string());
}

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

    /// Creates a new `LensSetTransform` from this lens and the given function.
    fn set_tx<F>(self, func: F) -> LensSetTransform<Self, F>
        where Self: Sized, F: Fn(&Self::Source) -> Self::Target
    {
        LensSetTransform { lens: self, func: func }
    }

    /// Creates a new `LensModifyTransform` from this lens and the given function.
    fn mod_tx<F>(self, func: F) -> LensModifyTransform<Self, F>
        where Self: Sized, F: Fn(&Self::Target) -> Self::Target
    {
        LensModifyTransform { lens: self, func: func }
    }

    /// Creates a new `LensIncrementTransform` from this lens.
    fn increment_tx(self) -> LensIncrementTransform<Self>
        where Self: Sized, Self::Target: PrimInt
    {
        // TODO: Ideally we would piggyback on `mod_tx` for this, but that would probably require removing `F` from
        // the type signature one way or another
        LensIncrementTransform { lens: self }
    }

    /// Creates a new `LensDecrementTransform` from this lens.
    fn decrement_tx(self) -> LensDecrementTransform<Self>
        where Self: Sized, Self::Target: PrimInt
    {
        LensDecrementTransform { lens: self }
    }

    /// Creates a new `LensNotTransform` from this lens.
    fn not_tx<T>(self) -> LensNotTransform<Self>
        where Self: Sized + Lens<Target=T>, T: Not<Output=T> + Clone
    {
        LensNotTransform { lens: self }
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

//
// Lens transforms
//

/// A transform that invokes a lens `set` operation with the output of the given function.
#[doc(hidden)]
pub struct LensSetTransform<L, F> {
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
#[doc(hidden)]
pub struct LensModifyTransform<L, F> {
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
#[doc(hidden)]
pub struct LensIncrementTransform<L> {
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
#[doc(hidden)]
pub struct LensDecrementTransform<L> {
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
#[doc(hidden)]
pub struct LensNotTransform<L> {
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

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct5 {
        enabled: bool
    }

    #[derive(Clone, Debug, PartialEq)]
    #[Lensed]
    struct Struct6 {
        left_enabled: bool,
        left: u32,
        right: u32
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

    #[test]
    fn a_lens_set_transform_should_work() {
        let tx = lens!(Struct1.foo).set_tx(|s| s.foo + 42);
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 42);
    }

    #[test]
    fn a_lens_modify_transform_should_work() {
        let tx = lens!(Struct1.foo).mod_tx(|foo| foo + 42);
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 42);
    }

    #[test]
    fn a_lens_increment_transform_should_work() {
        let tx = lens!(Struct1.foo).increment_tx();
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 1);
    }

    #[test]
    fn a_lens_decrement_transform_should_work() {
        let tx = lens!(Struct1.foo).decrement_tx();
        let s0 = Struct1 { foo: 42, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 41);
    }

    #[test]
    fn a_lens_not_transform_should_work() {
        let tx = lens!(Struct5.enabled).not_tx();
        let s0 = Struct5 { enabled: false };
        let s1 = tx.apply(s0);
        assert_eq!(s1.enabled, true);
    }

    #[test]
    fn lens_transform_composition_should_work() {
        let add_one = lens!(Struct1.foo).mod_tx(|foo| foo + 1);
        let add_two = lens!(Struct1.foo).mod_tx(|foo| foo + 2);
        let mul_two = lens!(Struct1.foo).mod_tx(|foo| foo * 2);
        let tx = composed_tx(add_one, composed_tx(add_two, mul_two));
        let s0 = Struct1 { foo: 0, bar: 0 };
        let s1 = tx.apply(s0);
        assert_eq!(s1.foo, 6);
    }
}
