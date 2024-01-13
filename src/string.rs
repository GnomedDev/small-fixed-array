use std::{borrow::Cow, cmp::PartialEq, fmt::Write as _, hash::Hash};

use crate::{
    array::FixedArray,
    inline::{get_heap_threshold, InlineString},
    length::{InvalidStrLength, SmallLen, ValidLength},
};

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
enum FixedStringRepr<LenT: ValidLength> {
    Heap(FixedArray<u8, LenT>),
    Inline(InlineString<LenT::InlineStrRepr>),
}

/// A fixed size String with length provided at creation denoted in [`ValidLength`], by default [`u32`].
///
/// See module level documentation for more information.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
pub struct FixedString<LenT: ValidLength = SmallLen>(FixedStringRepr<LenT>);

impl<LenT: ValidLength> FixedString<LenT> {
    #[must_use]
    pub fn new() -> Self {
        FixedString(FixedStringRepr::Inline(InlineString::from_str("")))
    }

    /// Returns the length of the [`FixedString`].
    #[must_use]
    pub fn len(&self) -> u32 {
        match &self.0 {
            FixedStringRepr::Heap(a) => a.len(),
            FixedStringRepr::Inline(a) => a.len(),
        }
    }

    /// Returns if the length is equal to 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        match &self.0 {
            // SAFETY: Self holds the type invariant that the array is UTF-8.
            FixedStringRepr::Heap(a) => unsafe { std::str::from_utf8_unchecked(a) },
            FixedStringRepr::Inline(a) => return a.as_str(),
        }
    }
}

impl<LenT: ValidLength> std::ops::DerefMut for FixedString<LenT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            // SAFETY: Self holds the type invariant that the array is UTF-8.
            FixedStringRepr::Heap(a) => unsafe { std::str::from_utf8_unchecked_mut(a.as_mut()) },
            FixedStringRepr::Inline(a) => return a.as_mut_str(),
        }
    }
}

impl<LenT: ValidLength> Default for FixedString<LenT> {
    fn default() -> Self {
        FixedString::new()
    }
}

impl<LenT: ValidLength> Clone for FixedString<LenT> {
    fn clone(&self) -> Self {
        match &self.0 {
            FixedStringRepr::Heap(a) => Self(FixedStringRepr::Heap(a.clone())),
            FixedStringRepr::Inline(a) => Self(FixedStringRepr::Inline(*a)),
        }
    }
}

impl<LenT: ValidLength> Hash for FixedString<LenT> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl<LenT: ValidLength> PartialEq for FixedString<LenT> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
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

impl<LenT: ValidLength> TryFrom<Box<str>> for FixedString<LenT> {
    type Error = InvalidStrLength;

    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        if value.len() <= get_heap_threshold::<LenT>() {
            let inner = InlineString::from_str(&value);
            return Ok(Self(FixedStringRepr::Inline(inner)));
        }

        match value.into_boxed_bytes().try_into() {
            Ok(val) => Ok(Self(FixedStringRepr::Heap(val))),
            Err(err) => Err(err
                .try_into()
                .expect("Box<str> -> Box<[u8]> should stay valid UTF8")),
        }
    }
}

#[cfg(any(feature = "log_using_log", feature = "log_using_tracing"))]
impl<LenT: ValidLength> From<String> for FixedString<LenT> {
    fn from(value: String) -> Self {
        if value.len() > get_heap_threshold::<LenT>() {
            let value = value.into_bytes().into();
            Self(FixedStringRepr::Heap(value))
        } else {
            Self(FixedStringRepr::Inline(InlineString::from_str(&value)))
        }
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for String {
    fn from(value: FixedString<LenT>) -> Self {
        match value.0 {
            // SAFETY: Self holds the type invariant that the array is UTF-8.
            FixedStringRepr::Heap(a) => unsafe { String::from_utf8_unchecked(a.into()) },
            FixedStringRepr::Inline(a) => a.as_str().to_string(),
        }
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for Cow<'static, str> {
    fn from(value: FixedString<LenT>) -> Self {
        Cow::Owned(value.into_string())
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

#[cfg(all(
    feature = "serde",
    any(feature = "log_using_log", feature = "log_using_tracing")
))]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_u8_roundtrip() {
        for i in 0..=u8::MAX {
            let original = "a".repeat(i.into()).into_boxed_str();
            let fixed = FixedString::<u8>::try_from(original).unwrap();

            assert!(fixed.bytes().all(|c| c == b'a'));
            assert_eq!(fixed.len(), i.into());
        }
    }

    #[test]
    fn check_sizes() {
        assert_eq!(std::mem::size_of::<Option<InlineString<[u8; 11]>>>(), 12);
        assert_eq!(std::mem::align_of::<Option<InlineString<[u8; 11]>>>(), 1);
        assert_eq!(std::mem::size_of::<Option<FixedArray<u8, u32>>>(), 12);
        assert_eq!(std::mem::align_of::<Option<FixedArray<u8, u32>>>(), 1);
        // This sucks!! I want to fix this, soon.... this should so niche somehow.
        assert_eq!(std::mem::size_of::<FixedStringRepr<u32>>(), 13);
        assert_eq!(std::mem::align_of::<FixedStringRepr<u32>>(), 1);
    }
}
