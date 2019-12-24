# pl-lens

[![Build Status][gh-actions-badge]][gh-actions-url]
[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[gh-actions-badge]: https://github.com/plausiblelabs/lens-rs/workflows/Build/badge.svg?event=push
[gh-actions-url]: https://github.com/plausiblelabs/lens-rs/actions?query=workflow%3ABuild+branch%3Amaster
[crates-badge]: https://img.shields.io/crates/v/pl-lens.svg
[crates-url]: https://crates.io/crates/pl-lens
[docs-badge]: https://docs.rs/pl-lens/badge.svg
[docs-url]: https://docs.rs/pl-lens
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE

This Rust library provides support for lenses, which are a mechanism in functional programming for focusing on a part of a complex data structure.

## Usage

Add a dependency (or two) to your `Cargo.toml`:

```toml
[dependencies]
pl-lens = "1.0"
```

Then, in your crate:

```rust
use pl_lens::*;
```

## Examples

A `Lens` can be used to transform a conceptually-immutable data structure by changing only a portion of the data.  Let's demonstrate with an example:

```rust
#[derive(Lenses)]
struct Address {
    street: String,
    city: String,
    postcode: String
}

#[derive(Lenses)]
struct Person {
    name: String,
    age: u8,
    address: Address
}

let p0 = Person {
    name: "Pop Zeus".to_string(),
    age: 58,
    address: Address {
        street: "123 Needmore Rd".to_string(),
        city: "Dayton".to_string(),
        postcode: "99999".to_string()
    }
};
assert_eq!(lens!(Person.name).get_ref(&p0), "Pop Zeus");
assert_eq!(lens!(Person.address.street).get_ref(&p0), "123 Needmore Rd");

let p1 = lens!(Person.address.street).set(p0, "666 Titus Ave".to_string());
assert_eq!(lens!(Person.name).get_ref(&p1), "Pop Zeus");
assert_eq!(lens!(Person.address.street).get_ref(&p1), "666 Titus Ave");
```

# License

`pl-lens` is distributed under an MIT license.  See LICENSE for more details.
