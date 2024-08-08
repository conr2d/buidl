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

use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{
	parse::Parse, punctuated::Punctuated, AttrStyle, Attribute, DeriveInput, Expr, Field, Ident,
	Meta, Token,
};

pub fn field_default_value(field: &Field) -> Option<Expr> {
	let outer_attrs = field.attrs.iter().filter(|attr| matches!(attr.style, AttrStyle::Outer));

	find_meta_item("buidl", outer_attrs, |meta| {
		if let Meta::NameValue(nv) = meta {
			if nv.path.is_ident("default") {
				return Some(nv.value)
			}
		}

		None
	})
}

pub fn crate_access(name: &str) -> Ident {
	let name = match crate_name(name) {
		Ok(FoundCrate::Itself) => name.to_string().replace("-", "_"),
		Ok(FoundCrate::Name(name)) => name,
		Err(_) => panic!("should have `{}` in dependencies", name),
	};

	Ident::new(&name, Span::call_site())
}

pub fn parse_top_attributes(input: &DeriveInput) -> Vec<Meta> {
	let mut outer_attrs = input.attrs.iter().filter(|attr| matches!(attr.style, AttrStyle::Outer));

	outer_attrs
		.find_map(|attr| {
			attr.path().is_ident("buidl").then(|| {
				let nested_meta =
					attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap();
				nested_meta.into_iter().collect()
			})
		})
		.unwrap_or_default()
}

pub fn parse_list_items(meta: &Meta) -> Vec<Meta> {
	match meta {
		Meta::List(list) => list
			.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
			.unwrap()
			.into_iter()
			.collect(),
		_ => Vec::new(),
	}
}

fn find_meta_item<'a, F, R, I, M>(kind: &str, mut itr: I, mut pred: F) -> Option<R>
where
	F: FnMut(M) -> Option<R> + Clone,
	I: Iterator<Item = &'a Attribute>,
	M: Parse,
{
	itr.find_map(|attr| attr.path().is_ident(kind).then(|| pred(attr.parse_args().ok()?)).flatten())
}
