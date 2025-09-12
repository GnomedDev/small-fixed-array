use alloc::{
    borrow::{Cow},
    boxed::Box,
    rc::Rc,
    string::{String},
    sync::Arc,
};
use core::{borrow::Borrow, hash::Hash, str::FromStr};

use crate::{
    array::FixedArray,
    inline::InlineString,
    length::{InvalidStrLength, SmallLen, ValidLength},
    r#static::StaticStr,
};

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
enum FixedStringRepr<LenT: ValidLength> {
    Static(StaticStr<LenT>),
    Heap(FixedArray<u8, LenT>),
    Inline(InlineString<LenT::InlineStrRepr>),
}

#[cold]
fn truncate_string(err: InvalidStrLength, max_len: usize) -> String {
    let mut value = String::from(err.get_inner());
    value.truncate(truncate_str(&value, max_len).len());
    value
}

#[cold]
fn truncate_str(string: &str, max_len: usize) -> &str {
    for len in (0..=max_len).rev() {
        if string.is_char_boundary(len) {
            return &string[..len];
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
        Self::from_static_trunc("")
    }

    pub(crate) fn new_inline(val: &str) -> Option<Self> {
        InlineString::from_str(val)
            .map(FixedStringRepr::Inline)
            .map(Self)
    }

    /// Converts a `&'static str` into a [`FixedString`].
    ///
    /// This method will not allocate, or copy the string data.
    ///
    /// See [`Self::from_string_trunc`] for truncation behaviour.
    pub fn from_static_trunc(val: &'static str) -> Self {
        Self(FixedStringRepr::Static(StaticStr::from_static_str(
            truncate_str(val, LenT::MAX.to_usize()),
        )))
    }

    /// Converts a string into a [`FixedString`], **truncating** if the value is larger than `LenT`'s maximum.
    ///
    /// If str is `& 'static str`, it is preferred to use [`Self::from_static_trunc`], which does not need to copy the data around.
    ///
    /// This allows for infallible conversion, but may be lossy in the case of a value above `LenT`'s max.
    /// For lossless fallible conversion use [`TryFrom`] or [`Self::try_from_string`].
    #[must_use]
    pub fn from_string_trunc<S>(str: S) -> Self
    where
        S: AsRef<str>,
        Box<str>: From<S>,
    {
        match Self::try_from_string(str) {
            Ok(val) => val,
            Err(err) => {
                Self::from_string_trunc::<String>(truncate_string(err, LenT::MAX.to_usize()))
            }
        }
    }

    /// Converts a string into a [`FixedString`].
    ///
    /// # Errors
    ///
    /// This function will return an error if str is longer than `LenT`'s maximum.
    pub fn try_from_string<S>(str: S) -> Result<Self, InvalidStrLength>
    where
        S: AsRef<str>,
        Box<str>: From<S>,
    {
        if let Some(inline) = Self::new_inline(str.as_ref()) {
            return Ok(inline);
        }

        match Box::<str>::from(str).into_boxed_bytes().try_into() {
            Ok(val) => Ok(Self(FixedStringRepr::Heap(val))),
            Err(err) => Err(
                // SAFETY: Box<str> -> Box<[u8]> -> Box<str> always works
                unsafe { InvalidStrLength::from_invalid_length_unchecked(err) },
            ),
        }
    }

    /// Returns the length of the [`FixedString`].
    #[must_use]
    pub fn len(&self) -> LenT {
        match &self.0 {
            FixedStringRepr::Heap(a) => a.len(),
            FixedStringRepr::Static(a) => a.len(),
            FixedStringRepr::Inline(a) => a.len().into(),
        }
    }

    /// Returns if the length is equal to 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == LenT::ZERO
    }

    /// Converts `&`[`FixedString`] to `&str`, this conversion can be performed by [`core::ops::Deref`].
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

    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_static(&self) -> bool {
        matches!(self, Self(FixedStringRepr::Static(_)))
    }
}

impl<LenT: ValidLength> core::ops::Deref for FixedString<LenT> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            // SAFETY: Self holds the type invariant that the array is UTF-8.
            FixedStringRepr::Heap(a) => unsafe { core::str::from_utf8_unchecked(a) },
            FixedStringRepr::Static(a) => a.as_str(),
            FixedStringRepr::Inline(a) => a.as_str(),
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
            FixedStringRepr::Static(a) => Self(FixedStringRepr::Static(*a)),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (&mut self.0, &source.0) {
            (FixedStringRepr::Heap(new), FixedStringRepr::Heap(src)) => new.clone_from(src),
            #[allow(clippy::assigning_clones)]
            _ => *self = source.clone(),
        }
    }
}

impl<LenT: ValidLength> Hash for FixedString<LenT> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
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

impl<LenT: ValidLength> core::cmp::PartialOrd for FixedString<LenT> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<LenT: ValidLength> core::cmp::Ord for FixedString<LenT> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<LenT: ValidLength> core::fmt::Display for FixedString<LenT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self)
    }
}

impl<LenT: ValidLength> core::fmt::Debug for FixedString<LenT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl<LenT: ValidLength> FromStr for FixedString<LenT> {
    type Err = InvalidStrLength;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        Self::try_from_string(val)
    }
}

macro_rules! try_from_impl {
    ($type:ty) => {
        impl<LenT: ValidLength> TryFrom<$type> for FixedString<LenT> {
            type Error = InvalidStrLength;

            fn try_from(value: $type) -> Result<Self, Self::Error> {
                Self::try_from_string(value)
            }
        }
    };
}

try_from_impl!(Box<str>);
try_from_impl!(String);
try_from_impl!(Cow<'_, str>);
try_from_impl!(&'_ str);

impl<LenT: ValidLength> TryFrom<Arc<str>> for FixedString<LenT> {
    type Error = InvalidStrLength;

    fn try_from(value: Arc<str>) -> Result<Self, Self::Error> {
        value.as_ref().try_into()
    }
}

impl<LenT: ValidLength> TryFrom<Rc<str>> for FixedString<LenT> {
    type Error = InvalidStrLength;

    fn try_from(value: Rc<str>) -> Result<Self, Self::Error> {
        value.as_ref().try_into()
    }
}

impl<LenT: ValidLength> From<char> for FixedString<LenT> {
    fn from(value: char) -> Self {
        use alloc::vec;

        if let Some(value) = InlineString::from_char(value) {
            return Self(FixedStringRepr::Inline(value));
        }

        let mut bytes = vec![0; value.len_utf8()].into_boxed_slice();

        value.encode_utf8(&mut bytes);

        let bytes = bytes
            .try_into()
            .expect("len_utf8 is at most 4, so it will fit in u8");

        Self(FixedStringRepr::Heap(bytes))
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for String {
    fn from(value: FixedString<LenT>) -> Self {
        Box::<str>::from(value).into()
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for Box<str> {
    fn from(value: FixedString<LenT>) -> Self {
        match value.0 {
            FixedStringRepr::Inline(a) => a.as_str().into(),
            FixedStringRepr::Static(a) => a.as_str().into(),
            // SAFETY: Self holds the type invariant that the array is UTF-8.
            FixedStringRepr::Heap(a) => unsafe { alloc::str::from_boxed_utf8_unchecked(a.into()) },
        }
    }
}

impl<'a, LenT: ValidLength> From<&'a FixedString<LenT>> for Cow<'a, str> {
    fn from(value: &'a FixedString<LenT>) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for Cow<'_, str> {
    fn from(value: FixedString<LenT>) -> Self {
        match value.0 {
            FixedStringRepr::Static(static_str) => Cow::Borrowed(static_str.as_str()),
            _ => Cow::Owned(value.into()),
        }
    }
}

impl<LenT: ValidLength> AsRef<str> for FixedString<LenT> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<LenT: ValidLength> Borrow<str> for FixedString<LenT> {
    fn borrow(&self) -> &str {
        self
    }
}

#[cfg(feature = "std")]
impl<LenT: ValidLength> AsRef<std::path::Path> for FixedString<LenT> {
    fn as_ref(&self) -> &std::path::Path {
        self.as_str().as_ref()
    }
}

#[cfg(feature = "std")]
impl<LenT: ValidLength> AsRef<std::ffi::OsStr> for FixedString<LenT> {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.as_str().as_ref()
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for Arc<str> {
    fn from(value: FixedString<LenT>) -> Self {
        Arc::from(Box::<str>::from(value))
    }
}

impl<LenT: ValidLength> From<FixedString<LenT>> for Rc<str> {
    fn from(value: FixedString<LenT>) -> Self {
        Rc::from(Box::<str>::from(value))
    }
}

#[cfg(feature = "to-arraystring")]
impl to_arraystring::ToArrayString for &FixedString<u8> {
    const MAX_LENGTH: usize = 255;
    type ArrayString = to_arraystring::ArrayString<255>;

    fn to_arraystring(self) -> Self::ArrayString {
        Self::ArrayString::from(self).unwrap()
    }
}

#[cfg(feature = "serde")]
impl<'de, LenT: ValidLength> serde::Deserialize<'de> for FixedString<LenT> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use core::marker::PhantomData;

        struct Visitor<LenT: ValidLength>(PhantomData<LenT>);

        impl<LenT: ValidLength> serde::de::Visitor<'_> for Visitor<LenT> {
            type Value = FixedString<LenT>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(formatter, "a string up to {} bytes long", LenT::MAX)
            }

            fn visit_str<E: serde::de::Error>(self, val: &str) -> Result<Self::Value, E> {
                FixedString::from_str(val).map_err(E::custom)
            }

            fn visit_string<E: serde::de::Error>(self, val: String) -> Result<Self::Value, E> {
                FixedString::try_from_string(val).map_err(E::custom)
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
    use core::fmt::Debug;

    use super::*;

    fn check_u8_roundtrip_generic(to_fixed: fn(String) -> FixedString<u8>) {
        for i in 0..=u8::MAX {
            let original = "a".repeat(i.into());
            let fixed = to_fixed(original);

            assert!(fixed.bytes().all(|c| c == b'a'));
            assert_eq!(fixed.len(), i);

            if !fixed.is_static() {
                assert_eq!(fixed.is_inline(), fixed.len() <= 9);
            }
        }
    }

    #[test]
    fn test_truncating_behaviour() {
        const STR: &str = "______________________________________________________________________________________________________________________________________________________________________________________________________________________________________________________________🦀";

        let string = FixedString::<u8>::from_static_trunc(STR);

        let str = std::str::from_utf8(string.as_bytes()).expect("is utf8");

        assert_eq!(str, string.as_str());
        assert_ne!(STR, str);
    }

    #[test]
    fn test_from_static_to_cow() {
        const STR: &str = "static string";

        let string = FixedString::<u8>::from_static_trunc(STR);

        let cow: std::borrow::Cow<'static, _> = string.into();

        assert_eq!(cow, STR);

        let std::borrow::Cow::Borrowed(string) = cow else {
            panic!("Expected borrowed string");
        };

        assert_eq!(string, STR);
    }

    #[test]
    fn check_u8_roundtrip() {
        check_u8_roundtrip_generic(|original| {
            FixedString::<u8>::try_from(original.into_boxed_str()).unwrap()
        });
    }

    #[test]
    fn check_u8_roundtrip_static() {
        check_u8_roundtrip_generic(|original| {
            let static_str = Box::leak(original.into_boxed_str());
            FixedString::from_static_trunc(static_str)
        });
    }

    #[test]
    #[cfg(feature = "serde")]
    fn check_u8_roundtrip_serde() {
        check_u8_roundtrip_generic(|original| {
            serde_json::from_str(&alloc::format!("\"{original}\"")).unwrap()
        });
    }

    #[test]
    #[cfg(feature = "to-arraystring")]
    fn check_u8_roundtrip_arraystring() {
        use to_arraystring::ToArrayString;

        check_u8_roundtrip_generic(|original| {
            FixedString::from_string_trunc(
                FixedString::from_string_trunc(original)
                    .to_arraystring()
                    .as_str(),
            )
        });
    }

    #[test]
    fn check_sizes() {
        type DoubleOpt<T> = Option<Option<T>>;

        assert_eq!(core::mem::size_of::<Option<InlineString<[u8; 11]>>>(), 12);
        assert_eq!(core::mem::align_of::<Option<InlineString<[u8; 11]>>>(), 1);
        assert_eq!(core::mem::size_of::<Option<FixedArray<u8, u32>>>(), 12);
        // https://github.com/rust-lang/rust/issues/119507
        assert_eq!(core::mem::size_of::<DoubleOpt<FixedArray<u8, u32>>>(), 13);
        assert_eq!(core::mem::align_of::<Option<FixedArray<u8, u32>>>(), 1);
        // This sucks!! I want to fix this, soon.... this should so niche somehow.
        assert_eq!(core::mem::size_of::<FixedStringRepr<u32>>(), 13);
        assert_eq!(core::mem::align_of::<FixedStringRepr<u32>>(), 1);
    }

    #[test]
    fn from_char_u8() {
        let s: FixedString<u8> = 'a'.into();
        assert_eq!(s.len(), 1);
        assert!(s.is_inline());

        let s: FixedString<u8> = '¼'.into();
        assert_eq!(s.len(), 2);
        assert!(s.is_inline());

        let s: FixedString<u8> = '⚡'.into();
        assert_eq!(s.len(), 3);
        assert!(s.is_inline());

        let s: FixedString<u8> = '🦀'.into();
        assert_eq!(s.len(), 4);
        #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
        assert!(s.is_inline());
    }

    #[test]
    fn from_char_u16() {
        let s: FixedString<u16> = 'a'.into();
        assert_eq!(s.len(), 1);
        assert!(s.is_inline());

        let s: FixedString<u16> = '¼'.into();
        assert_eq!(s.len(), 2);
        assert!(s.is_inline());

        let s: FixedString<u16> = '⚡'.into();
        assert_eq!(s.len(), 3);
        assert!(s.is_inline());

        let s: FixedString<u16> = '🦀'.into();
        assert_eq!(s.len(), 4);
        assert!(s.is_inline());
    }

    #[test]
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    fn from_char_u32() {
        let s: FixedString<u32> = 'a'.into();
        assert_eq!(s.len(), 1);
        assert!(s.is_inline());

        let s: FixedString<u32> = '¼'.into();
        assert_eq!(s.len(), 2);
        assert!(s.is_inline());

        let s: FixedString<u32> = '⚡'.into();
        assert_eq!(s.len(), 3);
        assert!(s.is_inline());

        let s: FixedString<u32> = '🦀'.into();
        assert_eq!(s.len(), 4);
        assert!(s.is_inline());
    }

    fn try_from_rountrip<LenT, S>(value: S)
    where
        LenT: ValidLength,
        FixedString<LenT>: TryFrom<S>,
        <FixedString<LenT> as TryFrom<S>>::Error: Debug,
        S: AsRef<str>,
        S: From<FixedString<LenT>>,
        Box<str>: From<S>,
    {
        let string = value.as_ref().to_string();

        let fixed_str: FixedString<LenT> = value.try_into().expect("Try into should work");

        assert_eq!(fixed_str, string);

        let value: S = fixed_str.into();

        assert_eq!(value.as_ref(), string);

        let fixed_str = FixedString::<LenT>::from_string_trunc(value);

        assert_eq!(fixed_str, string);

        let value: S = fixed_str.into();

        let fixed_str = FixedString::<LenT>::try_from_string(value).expect("try_from_string works");

        assert_eq!(fixed_str, string);
    }

    #[test]
    fn test_try_from_string() {
        let value = "Hello, world!";

        try_from_rountrip::<u8, String>(value.into());
        try_from_rountrip::<u16, String>(value.into());
        #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
        try_from_rountrip::<u32, String>(value.into());
    }

    #[test]
    fn test_try_from_boxed_str() {
        let value = "Hello, world!";

        try_from_rountrip::<u8, Box<str>>(value.into());
        try_from_rountrip::<u16, Box<str>>(value.into());
        #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
        try_from_rountrip::<u32, Box<str>>(value.into());
    }

    #[test]
    fn test_try_from_owned_cow_string() {
        let owned_cow: Cow<'static, str> = Cow::Owned("Hello, world!".into());

        #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
        try_from_rountrip::<u32, Cow<'static, str>>(owned_cow.clone());
        try_from_rountrip::<u16, Cow<'static, str>>(owned_cow.clone());
        try_from_rountrip::<u8, Cow<'static, str>>(owned_cow);
    }

    #[test]
    fn test_try_from_cow_string() {
        let owned_cow: Cow<'_, str> = Cow::Borrowed("Hello, world!");

        #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
        try_from_rountrip::<u32, Cow<'_, str>>(owned_cow.clone());
        try_from_rountrip::<u16, Cow<'_, str>>(owned_cow.clone());
        try_from_rountrip::<u8, Cow<'_, str>>(owned_cow);
    }
}
