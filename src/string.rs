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

#[cold]
fn truncate_string(err: InvalidStrLength, max_len: usize) -> String {
    let mut value = String::from(err.get_inner());
    for len in (0..=max_len).rev() {
        if value.is_char_boundary(len) {
            value.truncate(len);
            return value;
        }
    }

    unreachable!("Len 0 is a char boundary")
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

    /// # Panics
    /// Panics if val does not fit in the heap threshold.
    pub(crate) fn new_inline(val: &str) -> Self {
        Self(FixedStringRepr::Inline(InlineString::from_str(val)))
    }

    /// Converts a `&str` into a [`FixedString`], allocating if the value cannot fit "inline".
    ///
    /// This method will be more efficent if you would otherwise clone a [`String`] to convert into [`FixedString`],
    /// but should not be used in the case that [`String`] ownership could be transfered without reallocation.
    ///
    /// "Inline" refers to Small String Optimisation which allows for Strings with less than 9 to 11 characters
    /// to be stored without allocation, saving a pointer size and an allocation.
    ///
    /// See [`Self::from_string_trunc`] for truncation behaviour.
    #[must_use]
    pub fn from_str_trunc(val: &str) -> Self {
        if val.len() <= get_heap_threshold::<LenT>() {
            return Self::new_inline(val);
        }

        Self::from_string_trunc(val.to_owned())
    }

    /// Converts a [`String`] into a [`FixedString`], **truncating** if the value is larger than `LenT`'s maximum.
    ///
    /// This allows for infallible conversion, but may be lossy in the case of a value above `LenT`'s max.
    /// For lossless fallible conversion, convert to [`Box<str>`] using [`String::into_boxed_str`] and use [`TryFrom`].
    #[must_use]
    pub fn from_string_trunc(str: String) -> Self {
        match str.into_boxed_str().try_into() {
            Ok(val) => val,
            Err(err) => Self::from_string_trunc(truncate_string(err, LenT::MAX.to_usize())),
        }
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

    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_inline(&self) -> bool {
        matches!(self, Self(FixedStringRepr::Inline(_)))
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

impl<LenT: ValidLength> From<FixedString<LenT>> for String {
    fn from(value: FixedString<LenT>) -> Self {
        match value.0 {
            // SAFETY: Self holds the type invariant that the array is UTF-8.
            FixedStringRepr::Heap(a) => unsafe { String::from_utf8_unchecked(a.into()) },
            FixedStringRepr::Inline(a) => a.as_str().to_string(),
        }
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for Cow<'_, str> {
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

#[cfg(feature = "serde")]
impl<'de, LenT: ValidLength> serde::Deserialize<'de> for FixedString<LenT> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use std::marker::PhantomData;

        struct Visitor<LenT: ValidLength>(PhantomData<LenT>);

        impl<'de, LenT: ValidLength> serde::de::Visitor<'de> for Visitor<LenT> {
            type Value = FixedString<LenT>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a string up to {} bytes long", LenT::MAX)
            }

            fn visit_str<E: serde::de::Error>(self, val: &str) -> Result<Self::Value, E> {
                if val.len() <= get_heap_threshold::<LenT>() {
                    return Ok(FixedString::new_inline(val));
                }

                FixedString::try_from(Box::from(val)).map_err(E::custom)
            }

            fn visit_string<E: serde::de::Error>(self, val: String) -> Result<Self::Value, E> {
                FixedString::try_from(val.into_boxed_str()).map_err(E::custom)
            }
        }

        deserializer.deserialize_string(Visitor(PhantomData))
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

    fn check_u8_roundtrip_generic(to_fixed: fn(Box<str>) -> FixedString<u8>) {
        for i in 0..=u8::MAX {
            let original = "a".repeat(i.into()).into_boxed_str();
            let fixed = to_fixed(original);

            assert!(fixed.bytes().all(|c| c == b'a'));
            assert_eq!(fixed.len(), u32::from(i));
            assert_eq!(fixed.is_inline(), fixed.len() <= 8);
        }
    }
    #[test]
    fn check_u8_roundtrip() {
        check_u8_roundtrip_generic(|original| FixedString::<u8>::try_from(original).unwrap());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn check_u8_roundtrip_serde() {
        check_u8_roundtrip_generic(|original| {
            serde_json::from_str(&format!("\"{original}\"")).unwrap()
        });
    }

    #[test]
    fn check_sizes() {
        type DoubleOpt<T> = Option<Option<T>>;

        assert_eq!(std::mem::size_of::<Option<InlineString<[u8; 11]>>>(), 12);
        assert_eq!(std::mem::align_of::<Option<InlineString<[u8; 11]>>>(), 1);
        assert_eq!(std::mem::size_of::<Option<FixedArray<u8, u32>>>(), 12);
        // https://github.com/rust-lang/rust/issues/119507
        assert_eq!(std::mem::size_of::<DoubleOpt<FixedArray<u8, u32>>>(), 13);
        assert_eq!(std::mem::align_of::<Option<FixedArray<u8, u32>>>(), 1);
        // This sucks!! I want to fix this, soon.... this should so niche somehow.
        assert_eq!(std::mem::size_of::<FixedStringRepr<u32>>(), 13);
        assert_eq!(std::mem::align_of::<FixedStringRepr<u32>>(), 1);
    }
}
