//! A crate for [`FixedArray`] and [`FixedString`], types to provide a smaller memory footprint in exchange for:
//! - Immutablity, [`FixedArray`] and [`FixedString`] cannot be mutated without converting back to their expanded forms.
//! - Maximum length, [`FixedArray`] and [`FixedString`] have a length cap of `LenT::MAX` elements.
//!
//! These types provide cheap conversions to [`Vec`] and [`String`], to make up for most of these downsides, but it is
//! still not recommended to use these collections for mutated values as you will see a performance downside.
//!
//! These can be thought of as `Box<[T]>` and `Box<str>`, except the length is denoted as `LenT`, by default [`u32`].
//!
//! ## Features
//! - `nightly`: Speeds up [`FixedString::len`] for small strings, using `portable_simd`.
//! - `serde`: Provides [`serde`] implementations for [`FixedArray`] and [`FixedString`].
//! - `typesize`: Provides [`typesize`] implementations for [`FixedArray`] and [`FixedString`].
#![cfg_attr(feature = "nightly", feature(portable_simd))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::pedantic, clippy::as_conversions)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

mod array;
mod inline;
mod length;
mod string;
mod truncating_into;

pub use array::FixedArray;
pub use length::ValidLength;
pub use string::FixedString;
pub use truncating_into::TruncatingInto;
