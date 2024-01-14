use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};

use crate::inline::get_heap_threshold;

mod sealed {
    use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};

    pub trait LengthSealed {}
    impl LengthSealed for u8 {}
    impl LengthSealed for u16 {}
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    impl LengthSealed for u32 {}

    pub trait NonZeroSealed {}
    impl NonZeroSealed for NonZeroU8 {}
    impl NonZeroSealed for NonZeroU16 {}
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    impl NonZeroSealed for NonZeroU32 {}
}

#[derive(Debug)]
pub struct InvalidLength<T> {
    pub(crate) backtrace: std::backtrace::Backtrace,
    type_name: &'static str,
    original: Box<[T]>,
}

impl<T> InvalidLength<T> {
    #[cold]
    #[track_caller]
    pub(crate) fn new(type_name: &'static str, original: Box<[T]>) -> Self {
        Self {
            backtrace: std::backtrace::Backtrace::capture(),
            type_name,
            original,
        }
    }

    /// Returns the original Box<[T]> that could not be converted from.
    pub fn get_inner(self) -> Box<[T]> {
        self.original
    }
}

impl<T> std::fmt::Display for InvalidLength<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot fit {} into {}:\n\n{}",
            self.original.len(),
            self.type_name,
            self.backtrace
        )
    }
}

#[derive(Debug)]
pub struct InvalidStrLength {
    pub(crate) backtrace: std::backtrace::Backtrace,
    type_name: &'static str,
    original: Box<str>,
}

impl InvalidStrLength {
    /// Returns the original [`Box<str>`] that could not be converted from.
    pub fn get_inner(self) -> Box<str> {
        self.original
    }
}

impl std::fmt::Display for InvalidStrLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cannot fit {} into {}:\n\n{}",
            self.original.len(),
            self.type_name,
            self.backtrace
        )
    }
}

impl TryFrom<InvalidLength<u8>> for InvalidStrLength {
    type Error = std::str::Utf8Error;

    fn try_from(value: InvalidLength<u8>) -> Result<Self, Self::Error> {
        let original = if let Err(err) = std::str::from_utf8(&value.original) {
            return Err(err);
        } else {
            unsafe { std::str::from_boxed_utf8_unchecked(value.original) }
        };

        Ok(Self {
            original,
            type_name: value.type_name,
            backtrace: value.backtrace,
        })
    }
}

#[doc(hidden)]
pub trait NonZero<Int: ValidLength>:
    sealed::NonZeroSealed + Into<Int> + Sized + Copy + PartialEq + std::fmt::Debug
{
    fn new(val: Int) -> Option<Self>;
}

impl NonZero<u8> for NonZeroU8 {
    fn new(val: u8) -> Option<Self> {
        NonZeroU8::new(val)
    }
}

impl NonZero<u16> for NonZeroU16 {
    fn new(val: u16) -> Option<Self> {
        NonZeroU16::new(val)
    }
}

impl NonZero<u32> for NonZeroU32 {
    fn new(val: u32) -> Option<Self> {
        NonZeroU32::new(val)
    }
}

/// A sealed trait to represent valid lengths for a [`FixedArray`].
///
/// This is implemented on `u32` for non-16 bit platforms, and `u16` on all platforms.
///
/// [`FixedArray`]: `crate::array::FixedArray`
pub trait ValidLength: sealed::LengthSealed + Copy + TryFrom<usize> + Into<u32> {
    const ZERO: Self;
    const MAX: Self;
    const DANGLING: Self::NonZero;

    type NonZero: NonZero<Self>;
    #[cfg(feature = "typesize")]
    type InlineStrRepr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + typesize::TypeSize;
    #[cfg(not(feature = "typesize"))]
    type InlineStrRepr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default;

    #[must_use]
    fn to_usize(self) -> usize;

    #[must_use]
    fn from_usize(len: usize) -> Option<Self> {
        len.try_into().ok()
    }
}

impl ValidLength for u8 {
    const ZERO: Self = 0;
    const MAX: Self = Self::MAX;
    const DANGLING: Self::NonZero = Self::NonZero::MAX;

    type NonZero = NonZeroU8;
    type InlineStrRepr = [u8; get_heap_threshold::<Self>()];

    fn to_usize(self) -> usize {
        self.into()
    }
}

impl ValidLength for u16 {
    const ZERO: Self = 0;
    const MAX: Self = Self::MAX;
    const DANGLING: Self::NonZero = Self::NonZero::MAX;

    type NonZero = NonZeroU16;
    type InlineStrRepr = [u8; get_heap_threshold::<Self>()];

    fn to_usize(self) -> usize {
        self.into()
    }
}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
impl ValidLength for u32 {
    const ZERO: Self = 0;
    const MAX: Self = Self::MAX;
    const DANGLING: Self::NonZero = Self::NonZero::MAX;

    type NonZero = NonZeroU32;
    type InlineStrRepr = [u8; get_heap_threshold::<Self>()];

    fn to_usize(self) -> usize {
        self.try_into()
            .expect("u32 can fit into usize on platforms with pointer lengths of 32 and 64")
    }
}

#[cfg(target_pointer_width = "16")]
pub type SmallLen = u16;
#[cfg(not(target_pointer_width = "16"))]
pub type SmallLen = u32;
