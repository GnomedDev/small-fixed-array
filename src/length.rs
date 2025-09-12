use core::{
    fmt::{Debug, Display},
    num::{NonZeroU16, NonZeroU32, NonZeroU8},
};

use alloc::boxed::Box;

use crate::inline::get_heap_threshold;

mod sealed {
    use core::num::{NonZeroU16, NonZeroU32, NonZeroU8};

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
    type_name: &'static str,
    original: Box<[T]>,
}

impl<T> InvalidLength<T> {
    #[cold]
    #[track_caller]
    pub(crate) fn new(type_name: &'static str, original: Box<[T]>) -> Self {
        Self {
            type_name,
            original,
        }
    }

    /// Returns the original Box<[T]> that could not be converted from.
    pub fn get_inner(self) -> Box<[T]> {
        self.original
    }
}

#[cfg(feature = "std")]
impl<T: Debug> std::error::Error for InvalidLength<T> {}

impl<T> core::fmt::Display for InvalidLength<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Cannot fit {} into {}",
            self.original.len(),
            self.type_name,
        )
    }
}

#[derive(Debug)]
pub struct InvalidStrLength {
    type_name: &'static str,
    original: Box<str>,
}

impl InvalidStrLength {
    /// Returns the original [`Box<str>`] that could not be converted from.
    pub fn get_inner(self) -> Box<str> {
        self.original
    }

    pub(crate) unsafe fn from_invalid_length_unchecked(value: InvalidLength<u8>) -> Self {
        Self {
            type_name: value.type_name,
            original: unsafe { alloc::str::from_boxed_utf8_unchecked(value.original) },
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidStrLength {}

impl core::fmt::Display for InvalidStrLength {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Cannot fit {} into {}",
            self.original.len(),
            self.type_name,
        )
    }
}

#[doc(hidden)]
pub trait NonZero<Int: ValidLength>:
    sealed::NonZeroSealed + Into<Int> + Sized + Copy + PartialEq + Debug
{
    #[allow(unused)]
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
pub trait ValidLength:
    sealed::LengthSealed + Copy + Display + PartialEq + From<u8> + TryFrom<usize> + Into<u32>
{
    const ZERO: Self;
    const MAX: Self;
    #[deprecated = "will be removed in the next major release"]
    #[allow(deprecated)]
    const DANGLING: Self::NonZero;

    #[deprecated = "will be removed in the next major release"]
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
    #[allow(deprecated)]
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
    #[allow(deprecated)]
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
    #[allow(deprecated)]
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
