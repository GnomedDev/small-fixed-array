use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    impl Sealed for u32 {}
}

#[derive(Debug)]
pub struct InvalidLength<T> {
    pub(crate) backtrace: std::backtrace::Backtrace,
    type_name: &'static str,
    original: Box<[T]>,
}

impl<T> InvalidLength<T> {
    #[track_caller]
    fn new(type_name: &'static str, original: Box<[T]>) -> Self {
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

pub trait NonZero<T: sealed::Sealed>: Copy {
    fn new(val: T) -> Option<Self>;
    fn expand(self) -> T;
}

impl NonZero<u8> for NonZeroU8 {
    fn new(val: u8) -> Option<Self> {
        Self::new(val)
    }

    fn expand(self) -> u8 {
        self.get()
    }
}

impl NonZero<u16> for NonZeroU16 {
    fn new(val: u16) -> Option<Self> {
        Self::new(val)
    }

    fn expand(self) -> u16 {
        self.get()
    }
}

impl NonZero<u32> for NonZeroU32 {
    fn new(val: u32) -> Option<Self> {
        Self::new(val)
    }

    fn expand(self) -> u32 {
        self.get()
    }
}

/// A sealed trait to represent valid lengths for a [`FixedArray`].
///
/// This is implemented on `u32` for non-16 bit platforms, and `u16` on all platforms.
///
/// [`FixedArray`]: `crate::array::FixedArray`
pub trait ValidLength: sealed::Sealed + Default + Copy + TryFrom<usize> + Into<u32> {
    const MAX: usize;
    type NonZero: NonZero<Self>;

    /// # Errors
    ///
    /// Errors if the val's length cannot fit into Self.
    #[allow(clippy::type_complexity)]
    fn from_usize<T>(val: Box<[T]>) -> Result<Option<(Self::NonZero, Box<[T]>)>, InvalidLength<T>> {
        match val.len().try_into().map(Self::NonZero::new) {
            Ok(None) => Ok(None),
            Ok(Some(len)) => Ok(Some((len, val))),
            Err(_) => Err(InvalidLength::new(std::any::type_name::<Self>(), val)),
        }
    }

    fn to_usize(self) -> usize;
}

impl ValidLength for u8 {
    #[allow(clippy::as_conversions)] // Cannot use `.into()` in const.
    const MAX: usize = u8::MAX as usize;
    type NonZero = NonZeroU8;

    fn to_usize(self) -> usize {
        self.into()
    }
}

impl ValidLength for u16 {
    #[allow(clippy::as_conversions)] // Cannot use `.into()` in const.
    const MAX: usize = u16::MAX as usize;
    type NonZero = NonZeroU16;

    fn to_usize(self) -> usize {
        self.into()
    }
}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
impl ValidLength for u32 {
    #[allow(clippy::as_conversions)] // Cannot use `.into()` in const.
    const MAX: usize = u32::MAX as usize;
    type NonZero = NonZeroU32;

    fn to_usize(self) -> usize {
        self.try_into()
            .expect("u32 can fit into usize on platforms with pointer lengths of 32 and 64")
    }
}

#[cfg(target_pointer_width = "16")]
pub type SmallLen = u16;
#[cfg(not(target_pointer_width = "16"))]
pub type SmallLen = u32;
