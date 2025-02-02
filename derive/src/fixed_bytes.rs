// Copyright (C) Jeeyong Um <conr2d@proton.me>
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::utils;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DataStruct, DeriveInput, Expr, Field, Fields, Ident, Index, Type};

mod kw {
    pub const DERIVE: &str = "derive";
    pub const SKIP_DERIVE: &str = "skip_derive";

    pub const CLONE: &str = "Clone";
    pub const CODEC: &str = "Codec";
    pub const PASS_BY: &str = "PassBy";
    pub const SCALE: &str = "Scale";
    pub const SUBSTRATE: &str = "Substrate";
    pub const TYPE_INFO: &str = "TypeInfo";
}

pub const SUPPORTED_ATTRIBUTES: &[&str] = &[kw::DERIVE, kw::SKIP_DERIVE];
pub const SUPPORTED_DERIVES: &[&str] = &[kw::SCALE, kw::SUBSTRATE];
pub const SUPPORTED_SKIP_DERIVES: &[&str] = &[kw::CLONE, kw::CODEC, kw::PASS_BY, kw::TYPE_INFO];

trait Contains<T> {
    fn contains<U>(&self, value: U) -> bool
    where
        T: PartialEq<U>;
}
impl<T> Contains<T> for Vec<T> {
    fn contains<U>(&self, value: U) -> bool
    where
        T: PartialEq<U>,
    {
        self.iter().any(|item| item == &value)
    }
}

pub fn expand_fixed_bytes(input: DeriveInput) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let attrs = utils::parse_top_attributes(&input);
    let attrs_str = attrs
        .iter()
        .filter_map(|attr| attr.path().get_ident().map(|ident| ident.to_string()))
        .collect::<Vec<_>>();
    check_attributes("buidl", &attrs_str, SUPPORTED_ATTRIBUTES);

    let fields = check_struct_fields(&input.data);
    let field_names = struct_field_names(fields.clone());

    // first field should be a u8 array.
    let inner = &fields[0].ty;
    let size = check_array_size(inner);
    let data = field_names[0].clone();

    // build an initializer for the remaining fields.
    let remaining = (fields.len() > 1).then(|| {
        fields
            .into_iter()
            .enumerate()
            .skip(1)
            .fold(quote!(), |acc, (i, f)| {
                let ident = field_names[i].clone();
                match utils::field_default_value(f) {
                    Some(value) => quote! { #acc #ident: #value, },
                    None => quote! { #acc #ident: Default::default(), },
                }
            })
    });

    let derive = utils::find_list_strings(&attrs, "derive");
    check_attributes("derive", &derive, SUPPORTED_DERIVES);

    let skip_derive = utils::find_list_strings(&attrs, "skip_derive");
    check_attributes("skip_derive", &skip_derive, SUPPORTED_SKIP_DERIVES);

    let mut output = Vec::new();

    if !skip_derive.contains(kw::CLONE) {
        output.push(quote! {
            #[automatically_derived]
            impl #impl_generics Clone for #ty #ty_generics #where_clause {
                fn clone(&self) -> Self {
                    Self { #( #field_names: self.#field_names.clone(), )* }
                }
            }
        });
    }

    output.push(quote! {
		#[automatically_derived]
		impl #impl_generics PartialEq for #ty #ty_generics #where_clause {
			fn eq(&self, other: &Self) -> bool {
				self.#data.eq(&other.#data)
			}
		}

		#[automatically_derived]
		impl #impl_generics Eq for #ty #ty_generics #where_clause {}

		#[automatically_derived]
		impl #impl_generics PartialOrd for #ty #ty_generics #where_clause {
			fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
				self.#data.partial_cmp(&other.#data)
			}
		}

		#[automatically_derived]
		impl #impl_generics Ord for #ty #ty_generics #where_clause {
			fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
				self.#data.cmp(&other.#data)
			}
		}

		#[automatically_derived]
		impl #impl_generics From<#inner> for #ty #ty_generics #where_clause {
			fn from(value: #inner) -> Self {
				Self { #data: value, #remaining }
			}
		}

		#[automatically_derived]
		impl #impl_generics TryFrom<&[u8]> for #ty #ty_generics #where_clause {
			type Error = ();

			fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
				Ok(Self { #data: <#inner as TryFrom<&[u8]>>::try_from(value).map_err(|_| ())?, #remaining })
			}
		}

		#[automatically_derived]
		impl #impl_generics From<#ty #ty_generics> for #inner #where_clause {
			fn from(value: #ty #ty_generics) -> #inner {
				value.#data
			}
		}

		#[automatically_derived]
		impl #impl_generics AsRef<[u8]> for #ty #ty_generics #where_clause {
			fn as_ref(&self) -> &[u8] {
				&self.#data
			}
		}

		#[automatically_derived]
		impl #impl_generics AsMut<[u8]> for #ty #ty_generics #where_clause {
			fn as_mut(&mut self) -> &mut [u8] {
				&mut self.#data
			}
		}

		#[automatically_derived]
		impl #impl_generics ::core::hash::Hash for #ty #ty_generics #where_clause {
			fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) {
				self.#data.hash(state);
			}
		}

		#[automatically_derived]
		impl #impl_generics ::core::ops::Deref for #ty #ty_generics #where_clause {
			type Target = #inner;

			fn deref(&self) -> &Self::Target {
				&self.#data
			}
		}

		#[automatically_derived]
		impl #impl_generics ::core::ops::DerefMut for #ty #ty_generics #where_clause {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.#data
			}
		}
	});

    if derive.contains(kw::SUBSTRATE) {
        let sp_core = utils::crate_access("sp-core");

        output.push(quote! {
			#[automatically_derived]
			impl #impl_generics ::#sp_core::crypto::ByteArray for #ty #ty_generics #where_clause {
				const LEN: usize = #size;
			}

			#[automatically_derived]
			impl #impl_generics ::#sp_core::crypto::UncheckedFrom<#inner> for #ty #ty_generics #where_clause {
				fn unchecked_from(value: #inner) -> Self {
					Self::from(value)
				}
			}
		});

        if !skip_derive.contains(kw::CODEC) {
            let parity_scale_codec = utils::crate_access("parity-scale-codec");

            output.push(quote! {
				#[automatically_derived]
				impl #impl_generics ::#sp_core::crypto::FromEntropy for #ty #ty_generics #where_clause {
					fn from_entropy(input: &mut impl ::#parity_scale_codec::Input) -> Result<Self, ::#parity_scale_codec::Error> {
						let mut result = Self::from([0u8; #size]);
						input.read(result.as_mut())?;
						Ok(result)
					}
				}
			});
        }

        if !skip_derive.contains(kw::PASS_BY) {
            let sp_runtime_interface = utils::crate_access("sp-runtime-interface");

            output.push(quote! {
				#[automatically_derived]
				impl #impl_generics ::#sp_runtime_interface::pass_by::PassByInner for #ty #ty_generics #where_clause {
					type Inner = #inner;

					fn into_inner(self) -> Self::Inner {
						self.#data
					}

					fn inner(&self) -> &Self::Inner {
						&self.#data
					}

					fn from_inner(inner: Self::Inner) -> Self {
						Self::from(inner)
					}
				}

				#[automatically_derived]
				impl #impl_generics ::#sp_runtime_interface::pass_by::PassBy for #ty #ty_generics #where_clause {
					type PassBy = ::#sp_runtime_interface::pass_by::Inner<Self, #inner>;
				}
			});
        }
    }

    if derive.contains(kw::SUBSTRATE) || derive.contains(kw::SCALE) {
        if !skip_derive.contains(kw::CODEC) {
            let parity_scale_codec = utils::crate_access("parity-scale-codec");

            output.push(quote! {
				#[automatically_derived]
				impl #impl_generics ::#parity_scale_codec::Encode for #ty #ty_generics #where_clause {
					fn size_hint(&self) -> usize {
						self.#data.size_hint()
					}

					fn encode_to<__T: ::#parity_scale_codec::Output + ?Sized>(&self, dest: &mut __T) {
						self.#data.encode_to(dest)
					}
				}

				#[automatically_derived]
				impl #impl_generics ::#parity_scale_codec::EncodeLike for #ty #ty_generics #where_clause {}

				#[automatically_derived]
				impl #impl_generics ::#parity_scale_codec::Decode for #ty #ty_generics #where_clause {
					fn decode<__I: ::#parity_scale_codec::Input>(input: &mut __I) -> Result<Self, ::#parity_scale_codec::Error> {
						<#inner>::decode(input).map(Into::into)
					}
				}

				#[automatically_derived]
				impl #impl_generics ::#parity_scale_codec::MaxEncodedLen for #ty #ty_generics #where_clause {
					fn max_encoded_len() -> usize {
						<#inner>::max_encoded_len()
					}
				}
			});
        }

        if !skip_derive.contains(kw::TYPE_INFO) {
            let scale_info = utils::crate_access("scale-info");

            output.push(quote! {
                #[automatically_derived]
                impl #impl_generics ::#scale_info::TypeInfo for #ty #ty_generics #where_clause {
                    type Identity = #inner;

                    fn type_info() -> ::#scale_info::Type {
                        Self::Identity::type_info()
                    }
                }
            });
        }
    }

    quote! {
        #(#output)*
    }
}

fn check_struct_fields(data: &Data) -> Vec<&Field> {
    match data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(fields) => fields.named.iter().collect(),
            Fields::Unnamed(fields) => fields.unnamed.iter().collect(),
            Fields::Unit => panic!("no fields"),
        },
        _ => panic!("`struct` expected"),
    }
}

fn check_attributes(attr: &str, items: &[String], supported: &[&str]) {
    let unsupported = items
        .iter()
        .filter(|item| !supported.contains(&item.as_str()))
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        panic!("`{}` doesn't support {:?}", attr, unsupported);
    }
}

#[derive(Clone)]
enum FieldName {
    Named(Ident),
    Unnamed(Index),
}

impl ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldName::Named(ident) => ident.to_tokens(tokens),
            FieldName::Unnamed(index) => index.to_tokens(tokens),
        }
    }
}

impl From<(usize, &Field)> for FieldName {
    fn from((index, field): (usize, &Field)) -> Self {
        match field.ident {
            Some(ref ident) => FieldName::Named(ident.clone()),
            None => FieldName::Unnamed(Index::from(index)),
        }
    }
}

fn struct_field_names(fields: Vec<&Field>) -> Vec<FieldName> {
    fields
        .into_iter()
        .enumerate()
        .map(FieldName::from)
        .collect()
}

fn check_array_size(ty: &Type) -> Expr {
    if let Type::Array(arr) = ty {
        if let Type::Path(path) = &*arr.elem {
            if let Some(segment) = path.path.segments.first() {
                if segment.ident == "u8" {
                    return arr.len.clone();
                }
            }
        }
    }
    panic!("expected u8 array");
}
