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
//! - `serde`: Provides [`serde`] implementations for [`FixedArray`] and [`FixedString`].
//! - `typesize`: Provides [`typesize`] implementations for [`FixedArray`] and [`FixedString`].
//!
//! ## From implementations
//! [`From<Vec<T>>`]` for `[`FixedArray`] and [`From<String>`]` for `[`FixedString`] are only implemented if one of
//! `log_using_log` or `log_using_tracing` are enabled, as the implementations will `error` level log
//! if the Vec/String's length is too high for the provided `LenT` generic.
#![warn(clippy::pedantic, clippy::as_conversions)]
#![allow(clippy::module_name_repetitions)]

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
