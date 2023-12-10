use std::{cmp::PartialEq, fmt::Write as _, hash::Hash};

use crate::{
    array::FixedArray,
    length::{SmallLen, ValidLength},
};

/// A fixed size String with length provided at creation denoted in [`ValidLength`], by default [`SmallLen`].
///
/// See module level documentation for more information.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
pub struct FixedString<LenT: ValidLength = SmallLen>(FixedArray<u8, LenT>);

impl<LenT: ValidLength> FixedString<LenT> {
    #[must_use]
    pub fn new() -> Self {
        FixedString(FixedArray::default())
    }

    /// Returns the length of the [`FixedString`].
    #[must_use]
    pub fn len(&self) -> u32 {
        self.0.len()
    }

    /// Returns if the length is equal to 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts `&`[`FixedString`] to `&str`, this conversion can be performed by [`std::ops::Deref`].
    #[must_use]
    pub fn as_str(&self) -> &str {
        self
    }

    /// Converts [`FixedString`] to [`String`], this operation should be cheap.
    #[must_use]
    pub fn into_string(self) -> String {
        self.into()
    }
}

impl<LenT: ValidLength> std::ops::Deref for FixedString<LenT> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl<LenT: ValidLength> std::ops::DerefMut for FixedString<LenT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl<LenT: ValidLength> Default for FixedString<LenT> {
    fn default() -> Self {
        Self(FixedArray::empty())
    }
}

impl<LenT: ValidLength> Clone for FixedString<LenT> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<LenT: ValidLength> Hash for FixedString<LenT> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<LenT: ValidLength> PartialEq for FixedString<LenT> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<LenT: ValidLength> Eq for FixedString<LenT> {}

impl<LenT: ValidLength> PartialEq<String> for FixedString<LenT> {
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

impl<LenT: ValidLength> PartialEq<&str> for FixedString<LenT> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<LenT: ValidLength> PartialEq<str> for FixedString<LenT> {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl<LenT: ValidLength> PartialEq<FixedString<LenT>> for &str {
    fn eq(&self, other: &FixedString<LenT>) -> bool {
        other == self
    }
}

impl<LenT: ValidLength> PartialEq<FixedString<LenT>> for str {
    fn eq(&self, other: &FixedString<LenT>) -> bool {
        other == self
    }
}

impl<LenT: ValidLength> PartialEq<FixedString<LenT>> for String {
    fn eq(&self, other: &FixedString<LenT>) -> bool {
        other == self
    }
}

impl<LenT: ValidLength> std::cmp::PartialOrd for FixedString<LenT> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<LenT: ValidLength> std::cmp::Ord for FixedString<LenT> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<LenT: ValidLength> std::fmt::Display for FixedString<LenT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl<LenT: ValidLength> std::fmt::Debug for FixedString<LenT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl<LenT: ValidLength> From<String> for FixedString<LenT> {
    fn from(value: String) -> Self {
        let value = value.into_bytes();
        Self(value.into())
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for String {
    fn from(value: FixedString<LenT>) -> Self {
        unsafe { String::from_utf8_unchecked(value.0.into()) }
    }
}

impl<LenT: ValidLength> AsRef<str> for FixedString<LenT> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<LenT: ValidLength> AsRef<std::path::Path> for FixedString<LenT> {
    fn as_ref(&self) -> &std::path::Path {
        self.as_str().as_ref()
    }
}

impl<LenT: ValidLength> AsRef<std::ffi::OsStr> for FixedString<LenT> {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.as_str().as_ref()
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for std::sync::Arc<str> {
    fn from(value: FixedString<LenT>) -> Self {
        std::sync::Arc::from(value.into_string())
    }
}

#[cfg(feature = "serde")]
impl<'de, LenT: ValidLength> serde::Deserialize<'de> for FixedString<LenT> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(feature = "serde")]
impl<LenT: ValidLength> serde::Serialize for FixedString<LenT> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}
