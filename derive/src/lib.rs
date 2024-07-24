// Copyright (c) Jeeyong Um <conr2d@proton.me>
// SPDX-License-Identifier: MIT OR Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
/// - [`Hash`](core::hash::Hash)
/// - [`Deref`](core::ops::Deref)
///
/// ### Polkadot SDK traits (Optional)
/// - `Encode`, `EncodeLike`, `Decode`, `MaxEncodedLen`
/// - `TypeInfo`
/// - `ByteArray`, `UncheckedFrom`, `FromEntropy`
/// - `PassBy`, `PassByInner`
#[proc_macro_derive(FixedBytes, attributes(buidl))]
pub fn derive_fixed_bytes(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	expand_fixed_bytes(input).into()
}
