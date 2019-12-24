//
// Copyright (c) 2015 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use std::fmt;

/// An element in a `LensPath`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct LensPathElement {
    id: u64,
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
    pub elements: Vec<LensPathElement>,
}

impl LensPath {
    /// Creates a new `LensPath` with no elements.
    pub fn empty() -> LensPath {
        LensPath { elements: vec![] }
    }

    /// Creates a new `LensPath` with a single element.
    pub fn new(id: u64) -> LensPath {
        LensPath {
            elements: vec![LensPathElement { id: id }],
        }
    }

    /// Creates a new `LensPath` with a single index (for an indexed type such as `Vec`).
    pub fn from_index(index: usize) -> LensPath {
        LensPath {
            elements: vec![LensPathElement { id: index as u64 }],
        }
    }

    /// Creates a new `LensPath` with two elements.
    pub fn from_pair(id0: u64, id1: u64) -> LensPath {
        LensPath {
            elements: vec![LensPathElement { id: id0 }, LensPathElement { id: id1 }],
        }
    }

    /// Creates a new `LensPath` from a vector of element identifiers.
    pub fn from_vec(ids: Vec<u64>) -> LensPath {
        LensPath {
            elements: ids.iter().map(|id| LensPathElement { id: *id }).collect(),
        }
    }

    /// Creates a new `LensPath` that is the concatenation of the two paths.
    pub fn concat(lhs: LensPath, rhs: LensPath) -> LensPath {
        let mut elements = lhs.elements;
        elements.extend(&rhs.elements);
        LensPath { elements: elements }
    }
}

impl fmt::Debug for LensPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.elements
                .iter()
                .map(|elem| elem.id.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
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
