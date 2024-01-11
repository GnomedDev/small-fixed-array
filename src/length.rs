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

/// A sealed trait to represent valid lengths for a [`FixedArray`].
///
/// This is implemented on `u32` for non-16 bit platforms, and `u16` on all platforms.
///
/// [`FixedArray`]: `crate::array::FixedArray`
pub trait ValidLength: sealed::Sealed + Default + Copy + TryFrom<usize> + Into<u32> {
    const ZERO: Self;
    const MAX: usize;

    #[must_use]
    fn to_usize(self) -> usize;

    #[must_use]
    fn from_usize(len: usize) -> Option<Self> {
        len.try_into().ok()
    }
}

impl ValidLength for u8 {
    const ZERO: Self = 0;
    #[allow(clippy::as_conversions)] // Cannot use `.into()` in const.
    const MAX: usize = u8::MAX as usize;

    fn to_usize(self) -> usize {
        self.into()
    }
}

impl ValidLength for u16 {
    const ZERO: Self = 0;
    #[allow(clippy::as_conversions)] // Cannot use `.into()` in const.
    const MAX: usize = u16::MAX as usize;

    fn to_usize(self) -> usize {
        self.into()
    }
}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
impl ValidLength for u32 {
    const ZERO: Self = 0;
    #[allow(clippy::as_conversions)] // Cannot use `.into()` in const.
    const MAX: usize = u32::MAX as usize;

    fn to_usize(self) -> usize {
        self.try_into()
            .expect("u32 can fit into usize on platforms with pointer lengths of 32 and 64")
    }
}

#[cfg(target_pointer_width = "16")]
pub type SmallLen = u16;
#[cfg(not(target_pointer_width = "16"))]
pub type SmallLen = u32;
