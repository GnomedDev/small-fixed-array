use std::fmt::Write as _;

use crate::FixedArray;

/// A fixed size String with length provided at creation denoted in [`u32`].
///
/// See module level documentation for more information.
#[derive(Default, Clone, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
pub struct FixedString(FixedArray<u8>);

impl FixedString {
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

impl std::ops::Deref for FixedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl std::ops::DerefMut for FixedString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl std::cmp::PartialEq<&str> for FixedString {
    fn eq(&self, other: &&str) -> bool {
        (&self.as_str()).eq(other)
    }
}

impl std::cmp::PartialEq<str> for FixedString {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl std::cmp::PartialEq<FixedString> for &str {
    fn eq(&self, other: &FixedString) -> bool {
        other == self
    }
}

impl std::cmp::PartialEq<FixedString> for str {
    fn eq(&self, other: &FixedString) -> bool {
        other == self
    }
}

impl std::cmp::PartialOrd for FixedString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for FixedString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::fmt::Display for FixedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl std::fmt::Debug for FixedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl From<String> for FixedString {
    fn from(value: String) -> Self {
        let value = value.into_bytes();
        Self(value.into())
    }
}

impl From<FixedString> for String {
    fn from(value: FixedString) -> Self {
        unsafe { String::from_utf8_unchecked(value.0.into()) }
    }
}

impl From<FixedString> for std::sync::Arc<str> {
    fn from(value: FixedString) -> Self {
        let boxed_array = value.0.into_boxed_slice();
        let boxed_str = unsafe { std::str::from_boxed_utf8_unchecked(boxed_array) };
        std::sync::Arc::from(boxed_str)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FixedString {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for FixedString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}
