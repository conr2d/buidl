// Copyright (c) Jeeyong Um <conr2d@proton.me>
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Custom derive for buidl.

use crate::fixed_bytes::expand_fixed_bytes;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod fixed_bytes;
mod utils;

/// Derives traits for a fixed-size byte array.
///
/// This macro derives traits for structs or tuples where the first field is a fixed-size array of
/// `u8`. The target type can have additional fields, which will be initialized with their default
/// values. A custom default value can be specified by using the `#[buidl(default = $expr)]`
/// attribute.
///
/// ```
/// # use buidl_derive::FixedBytes;
/// #[derive(FixedBytes)]
/// struct Bytes32 {
///   data: [u8; 32],
///   #[buidl(default = true)]
///   dirty: bool,
/// }
/// ```
///
/// ## The derived traits
///
/// ### Core traits
/// - [`Clone`]
/// - [`PartialEq`], [`Eq`], [`PartialOrd`], [`Ord`]
/// - [`From`], [`TryFrom`], [`AsRef`], [`AsMut`]
/// - [`Hash`]
/// - [`Deref`], [`DerefMut`]
///
/// [`Hash`]: core::hash::Hash
/// [`Deref`]: core::ops::Deref
/// [`DerefMut`]: core::ops::DerefMut
///
/// ### Polkadot SDK traits (Optional)
///
/// Polkadot SDK traits are derived when the `substrate` attribute is specified.
///
/// ```
/// # use buidl_derive::FixedBytes;
/// # use core::marker::PhantomData;
/// #[derive(FixedBytes)]
/// #[buidl(substrate)]
/// struct CryptoBytes<const N: usize, T = ()>([u8; N], PhantomData<fn() -> T>);
/// ```
///
/// - [`ByteArray`], [`UncheckedFrom`], [`FromEntropy`]
/// - [`PassBy`], [`PassByInner`]
/// - [`Encode`], [`EncodeLike`], [`Decode`], `MaxEncodedLen`
/// - [`TypeInfo`]
///
/// [`ByteArray`]: https://docs.rs/sp-core/32.0.0/sp_core/crypto/trait.ByteArray.html
/// [`UncheckedFrom`]: https://docs.rs/sp-core/32.0.0/sp_core/crypto/trait.UncheckedFrom.html
/// [`FromEntropy`]: https://docs.rs/sp-core/32.0.0/sp_core/crypto/trait.FromEntropy.html
/// [`PassBy`]: https://docs.rs/sp-runtime-interface/27.0.0/sp_runtime_interface/pass_by/trait.PassBy.html
/// [`PassByInner`]: https://docs.rs/sp-runtime-interface/27.0.0/sp_runtime_interface/pass_by/trait.PassByInner.html
/// [`Decode`]: https://docs.rs/parity-scale-codec/3.6.12/parity_scale_codec/trait.Decode.html
/// [`Encode`]: https://docs.rs/parity-scale-codec/3.6.12/parity_scale_codec/trait.Encode.html
/// [`EncodeLike`]: https://docs.rs/parity-scale-codec/3.6.12/parity_scale_codec/trait.EncodeLike.html
/// [`TypeInfo`]: https://docs.rs/scale-info/2.11.3/scale_info/trait.TypeInfo.html
#[proc_macro_derive(FixedBytes, attributes(buidl))]
pub fn derive_fixed_bytes(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	expand_fixed_bytes(input).into()
}
