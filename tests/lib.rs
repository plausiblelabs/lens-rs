//
// Copyright (c) 2016-2019 Plausible Labs Cooperative, Inc.
// All rights reserved.
//

use pl_lens::Lenses;

#[derive(Lenses)]
struct Address {
    street: String,
    city: String,
    postcode: String,
    #[leaf] names: Vec<String>
}

#[derive(Lenses)]
struct Person {
    name: String,
    age: u8,
    address: Address,
}

#[test]
fn a_simple_nested_data_structure_should_be_lensable() {
    use pl_lens::{lens, Lens, RefLens};

    let p0 = Person {
        name: "Pop Zeus".to_string(),
        age: 58,
        address: Address {
            street: "123 Needmore Rd".to_string(),
            city: "Dayton".to_string(),
            postcode: "99999".to_string(),
            names: vec!["a".into(),"vec".into()],
        },
    };
    assert_eq!(lens!(Person.name).get_ref(&p0), "Pop Zeus");
    assert_eq!(lens!(Person.address.street).get_ref(&p0), "123 Needmore Rd");

    let p1 = lens!(Person.address.street).set(p0, "666 Titus Ave".to_string());
    assert_eq!(lens!(Person.name).get_ref(&p1), "Pop Zeus");
    assert_eq!(lens!(Person.address.street).get_ref(&p1), "666 Titus Ave");
}
