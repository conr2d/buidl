// Copyright (C) Jeeyong Um <conr2d@proton.me>
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use buidl_derive::FixedBytes;
use std::collections::HashSet;

macro_rules! test_derive_traits {
    ($x:ident, $y:tt) => {
        let s = $x { $y: [1, 2, 3, 4] };

        // Eq
        let s2 = $x { $y: [1, 2, 3, 4] };
        assert_eq!(s, s2);

        // Ord
        let s2 = $x { $y: [2, 2, 2, 4] };
        assert!(s < s2);

        // From
        let a: [u8; 4] = [1, 2, 3, 4];
        let s2 = $x::from(a);
        assert_eq!(s, s2);
        assert_eq!(a, <[u8; 4]>::from(s2));

        // TryFrom
        let s2 = $x::try_from(&a[..]).unwrap();
        assert_eq!(s, s2);

        // AsRef
        assert_eq!(s.as_ref(), &a[..]);

        // AsMut
        let mut s2 = s.clone();
        s2.as_mut()[2] = 2;
        assert_eq!(s2, $x::from([1, 2, 2, 4]));

        // Clone
        let s2 = s.clone();
        assert_eq!(s, s2);

        // Hash
        let mut map = HashSet::<$x>::new();
        map.insert(s.clone());

        // Deref
        assert_eq!(*s, a);
    };
}

#[test]
fn derive_struct() {
    #[allow(unused)]
    #[derive(FixedBytes, Debug)]
    struct Struct {
        inner: [u8; 4],
    }

    test_derive_traits!(Struct, inner);
}

#[test]
fn derive_tuple() {
    #[allow(unused)]
    #[derive(FixedBytes, Debug)]
    struct Tuple([u8; 4]);

    test_derive_traits!(Tuple, 0);
}

#[test]
fn derive_substrate() {
    use codec::{Encode, MaxEncodedLen};

    #[allow(unused)]
    #[derive(FixedBytes, Debug)]
    #[buidl(derive(Substrate))]
    struct Substrate([u8; 4]);

    let s = Substrate::from([1, 2, 3, 4]);
    let encoded = s.encode();

    assert_eq!(encoded, vec![1, 2, 3, 4]);
    assert_eq!(Substrate::max_encoded_len(), 4);
}

#[test]
fn additional_fields() {
    #[allow(unused)]
    #[derive(FixedBytes, Debug)]
    struct Struct {
        inner: [u8; 4],
        #[buidl(default = 42)]
        other: u32,
    }

    let s = Struct::from([1, 2, 3, 4]);
    assert_eq!(s.other, 42);
}
