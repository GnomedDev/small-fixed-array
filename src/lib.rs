//! A crate for [`FixedArray`] and [`FixedString`], types to provide a smaller memory footprint in exchange for:
//! - Immutablity, [`FixedArray`] and [`FixedString`] cannot be mutated without converting back to their expanded forms.
//! - Maximum length, [`FixedArray`] and [`FixedString`] have a length cap of [`u32::MAX`] elements.
//!
//! These types provide cheap conversions to [`Vec`] and [`String`], to make up for most of these downsides, but it is
//! still not recommended to use these collections for mutated values as you will see a performance downside.
//!
//! These can be thought of as `Box<[T]>` and `Box<str>`, except the length is denoted as a [`u32`].
//!
//! ## Features
//! - `serde`: Provides [`serde`] implementations for [`FixedArray`] and [`FixedString`].
//! - `typesize`: Provides [`typesize`] implementations for [`FixedArray`] and [`FixedString`].
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod array;
mod string;
// Internal only!
mod non_empty_array;

pub use array::FixedArray;
pub use string::FixedString;
