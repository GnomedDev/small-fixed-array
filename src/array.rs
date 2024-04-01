use alloc::{borrow::Cow, boxed::Box, sync::Arc, vec::Vec};
use core::{fmt::Debug, hash::Hash, mem::ManuallyDrop, ptr::NonNull};

use crate::length::{InvalidLength, NonZero, SmallLen, ValidLength};

#[cold]
fn truncate_vec<T>(err: InvalidLength<T>, max_len: usize) -> Vec<T> {
    let mut value = Vec::from(err.get_inner());
    value.truncate(max_len);
    value
}

/// A fixed size array with length provided at creation denoted in a [`ValidLength`], by default [`u32`].
///
/// See module level documentation for more information.
#[repr(packed)]
pub struct FixedArray<T, LenT: ValidLength = SmallLen> {
    ptr: NonNull<T>,
    len: LenT::NonZero,
}

impl<T, LenT: ValidLength> FixedArray<T, LenT> {
    /// Alias to [`FixedArray::empty`].
    #[must_use]
    pub fn new() -> Self {
        Self::empty()
    }

    /// Creates a new, empty [`FixedArray`] that cannot be pushed to.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: LenT::DANGLING,
        }
    }

    /// # Safety
    /// - `len` must be equal to `ptr.len()`
    unsafe fn from_box(ptr: Box<[T]>, len: LenT) -> Self {
        let len = LenT::NonZero::new(len).unwrap_or(LenT::DANGLING);

        // If the length was 0, the above `unwrap_or` has just set the value to `LenT::DANGLING`.
        // If the length was not 0, the invariant is held by the caller.
        Self::from_box_with_nonzero(ptr, len)
    }

    /// # Safety
    /// If the slice is empty:
    /// - `len` must be equal to `LenT::DANGLING`
    ///
    /// If the slice is not empty:
    /// - `len` must be equal to `ptr.len()`
    #[must_use]
    unsafe fn from_box_with_nonzero(ptr: Box<[T]>, len: LenT::NonZero) -> Self {
        #[cfg(debug_assertions)]
        if ptr.is_empty() {
            assert_eq!(len, LenT::DANGLING);
        } else {
            assert_eq!(len.into().to_usize(), ptr.len());
        }

        let array_ptr = Box::into_raw(ptr).cast::<T>();
        Self {
            ptr: NonNull::new(array_ptr).expect("Box ptr != nullptr"),
            len,
        }
    }

    /// Converts [`Vec<T>`] into [`FixedArray<T>`] while truncating the vector if above the maximum size of `LenT`.
    #[must_use]
    pub fn from_vec_trunc(vec: Vec<T>) -> Self {
        match vec.into_boxed_slice().try_into() {
            Ok(v) => v,
            Err(err) => Self::from_vec_trunc(truncate_vec(err, LenT::MAX.to_usize())),
        }
    }

    /// Returns the length of the [`FixedArray`].
    #[must_use]
    pub fn len(&self) -> LenT {
        if self.is_empty() {
            LenT::ZERO
        } else {
            self.len.into()
        }
    }

    /// Returns if the length is equal to 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        ({ self.ptr }) == NonNull::dangling()
    }

    /// Converts [`FixedArray<T>`] to [`Vec<T>`], this operation should be cheap.
    #[must_use]
    pub fn into_vec(self) -> Vec<T> {
        self.into()
    }

    /// Converts [`FixedArray<T>`] to `Box<[T]>`, this operation should be cheap.
    #[must_use]
    pub fn into_boxed_slice(self) -> Box<[T]> {
        self.into()
    }

    /// Converts `&`[`FixedArray<T>`] to `&[T]`, this conversion can be performed by [`core::ops::Deref`].
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Converts `&mut `[`FixedArray<T>`] to `&mut [T]`, this conversion can be performed by [`core::ops::DerefMut`].
    #[must_use]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }

    /// Converts the [`FixedArray`] to it's original [`Box<T>`].
    ///
    /// # Safety
    /// `self` must never be used again, and it is highly recommended to wrap in [`ManuallyDrop`] before calling.
    pub(crate) unsafe fn as_box(&mut self) -> Box<[T]> {
        let slice = self.as_slice_mut();

        // SAFETY: `self` has been derived from `Box<[T]>`
        unsafe { Box::from_raw(slice) }
    }
}

unsafe impl<T: Send, LenT: ValidLength> Send for FixedArray<T, LenT> {}
unsafe impl<T: Sync, LenT: ValidLength> Sync for FixedArray<T, LenT> {}

impl<T, LenT: ValidLength> core::ops::Deref for FixedArray<T, LenT> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        // SAFETY: `self.ptr` and `self.len` are both valid and derived from `Box<[T]>`.
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len().to_usize()) }
    }
}

impl<T, LenT: ValidLength> core::ops::DerefMut for FixedArray<T, LenT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: `self.ptr` and `self.len` are both valid and derived from `Box<[T]>`.
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len().to_usize()) }
    }
}

impl<T, LenT: ValidLength> Drop for FixedArray<T, LenT> {
    fn drop(&mut self) {
        // SAFETY: We never use `self` again, and we are in the drop impl.
        unsafe { self.as_box() };
    }
}

impl<T, LenT: ValidLength> Default for FixedArray<T, LenT> {
    /// Creates a new, empty [`FixedArray`] that cannot be pushed to.
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: Clone, LenT: ValidLength> Clone for FixedArray<T, LenT> {
    fn clone(&self) -> Self {
        let ptr = self.as_slice().to_vec().into_boxed_slice();

        // SAFETY: The Box::from cannot make the length mismatch.
        unsafe { Self::from_box_with_nonzero(ptr, self.len) }
    }

    #[allow(clippy::assigning_clones)]
    fn clone_from(&mut self, source: &Self) {
        if self.len() == source.len() {
            self.clone_from_slice(source);
        } else {
            *self = source.clone();
        }
    }
}

impl<T, LenT: ValidLength> core::ops::Index<LenT> for FixedArray<T, LenT> {
    type Output = T;
    fn index(&self, index: LenT) -> &Self::Output {
        &self.as_slice()[index.to_usize()]
    }
}

impl<T, LenT: ValidLength> core::ops::IndexMut<LenT> for FixedArray<T, LenT> {
    fn index_mut(&mut self, index: LenT) -> &mut Self::Output {
        &mut self.as_slice_mut()[index.to_usize()]
    }
}

impl<T: Hash, LenT: ValidLength> Hash for FixedArray<T, LenT> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T: PartialEq, LenT: ValidLength> PartialEq for FixedArray<T, LenT> {
    // https://github.com/rust-lang/rust-clippy/issues/12154
    #[allow(
        unknown_lints,
        unconditional_recursion,
        clippy::unconditional_recursion
    )]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T: Eq, LenT: ValidLength> Eq for FixedArray<T, LenT> {}

impl<T: Debug, LenT: ValidLength> Debug for FixedArray<T, LenT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <[T] as Debug>::fmt(self, f)
    }
}

impl<T, LenT: ValidLength> IntoIterator for FixedArray<T, LenT> {
    type Item = <Vec<T> as IntoIterator>::Item;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

impl<'a, T, LenT: ValidLength> IntoIterator for &'a FixedArray<T, LenT> {
    type Item = <&'a [T] as IntoIterator>::Item;
    type IntoIter = <&'a [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T, LenT: ValidLength> IntoIterator for &'a mut FixedArray<T, LenT> {
    type Item = <&'a mut [T] as IntoIterator>::Item;
    type IntoIter = <&'a mut [T] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice_mut().iter_mut()
    }
}

impl<T, LenT: ValidLength> From<FixedArray<T, LenT>> for Box<[T]> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        let mut value = ManuallyDrop::new(value);

        // SAFETY: We don't use value again, and it is ManuallyDrop.
        unsafe { value.as_box() }
    }
}

impl<T, LenT: ValidLength> From<FixedArray<T, LenT>> for Vec<T> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        value.into_boxed_slice().into_vec()
    }
}

impl<T: Clone, LenT: ValidLength> From<FixedArray<T, LenT>> for Cow<'_, [T]> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        Cow::Owned(value.into_vec())
    }
}

impl<T, LenT: ValidLength> From<FixedArray<T, LenT>> for Arc<[T]> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        Arc::from(value.into_boxed_slice())
    }
}

impl<T, LenT: ValidLength> TryFrom<Box<[T]>> for FixedArray<T, LenT> {
    type Error = InvalidLength<T>;
    fn try_from(boxed_array: Box<[T]>) -> Result<Self, Self::Error> {
        let Some(len) = LenT::from_usize(boxed_array.len()) else {
            return Err(InvalidLength::new(
                core::any::type_name::<LenT>(),
                boxed_array,
            ));
        };

        // SAFETY: `len` was derived from the box length.
        Ok(unsafe { Self::from_box(boxed_array, len) })
    }
}

macro_rules! impl_array_from {
    ($($N:expr),*) => {
        $(
            impl<T, LenT: ValidLength> From<[T; $N]> for FixedArray<T, LenT> {
                fn from(val: [T; $N]) -> Self {
                    Self::try_from(Box::from(val))
                        .unwrap_or_else(|_| unreachable!(concat!($N, " should be less than {}"), LenT::MAX))
                }
            }
        )*
    };
}

impl_array_from!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

impl<T, LenT: ValidLength> AsRef<[T]> for FixedArray<T, LenT> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

#[cfg(feature = "serde")]
impl<'de, T, LenT> serde::Deserialize<'de> for FixedArray<T, LenT>
where
    T: serde::Deserialize<'de>,
    LenT: ValidLength,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_from(Box::<[T]>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl<T, LenT> serde::Serialize for FixedArray<T, LenT>
where
    T: serde::Serialize,
    LenT: ValidLength,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "typesize")]
impl<T: typesize::TypeSize, LenT: ValidLength> typesize::TypeSize for FixedArray<T, LenT> {
    fn extra_size(&self) -> usize {
        self.iter().map(T::get_size).sum()
    }
}
