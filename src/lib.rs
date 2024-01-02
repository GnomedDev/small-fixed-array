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
#![warn(clippy::pedantic, clippy::as_conversions)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(
    not(any(feature = "log_using_log", feature = "log_using_tracing")),
    deprecated = "Please pick a logging framework using the `log_using_log` or `log_using_tracing` features!"
)]

mod array;
mod length;
mod logging;
mod string;

pub use array::FixedArray;
pub use length::ValidLength;
pub use string::FixedString;

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use crate::FixedString;

    #[test]
    fn niche_test() {
        assert_eq!(size_of::<FixedString>(), size_of::<Option<FixedString>>());
    }
}
