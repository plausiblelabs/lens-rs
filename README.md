# lens-rs

This Rust library provides support for lenses, which are a mechanism in functional programming for focusing on a part of a complex data structure.

## Usage

Add a dependency (or two) to your `Cargo.toml`:

```toml
[dependencies]
lens = { git = "https://opensource.plausible.coop/src/scm/rc/lens-rs.git" }
lens_macros = { git = "https://opensource.plausible.coop/src/scm/rc/lens-rs.git" }
```

Then, in your crate:

```rust
// The following allows for using macros defined in the separate lens_macros crate.
#![feature(plugin, custom_attribute)]
#![plugin(lens_macros)]

#[macro_use]
extern crate lens;

use lens::*;
```

## Examples

A `Lens` can be used to transform a conceptually-immutable data structure by changing only a portion of the data.  Let's demonstrate with an example:

```rust
#[Lensed]
struct Address {
    street: String,
    city: String,
    postcode: String
}

#[Lensed]
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

The `lens` crate also offers a simple `Transform` API and some built-in functions that can be used in conjunction with the `Lens` API to modify a data structure through a series of transforms.  For example:

```rust
#[Lensed]
struct Header {
    count: u16
}

#[Lensed]
struct Packet {
    header: Header,
    data: Vec<u8>
}

let count_lens = || { lens!(Packet.header.count) };
let add_one = increment_tx(count_lens());
let add_two = mod_tx(count_lens(), |c| c + 2);
let mul_two = mod_tx(count_lens(), |c| c * 2);
let tx = compose_tx!(add_one, add_two, mul_two);

let p0 = Packet { header: Header { count: 0 }, data: vec![] };
let p1 = tx.apply(p0);
assert_eq!(p1.header.count, 6);
let p2 = tx.apply(p1);
assert_eq!(p2.header.count, 18);
```

# License

`lens-rs` is distributed under an MIT license.  See LICENSE for more details.
