use std::{mem::ManuallyDrop, ptr::NonNull};

use crate::length::{InvalidLength, NonZero, ValidLength};

#[repr(packed)]
pub(crate) struct NonEmptyFixedArray<T, LenT: ValidLength> {
    ptr: NonNull<T>,
    len: LenT::NonZero,
}

impl<T, LenT: ValidLength> NonEmptyFixedArray<T, LenT> {
    pub(crate) fn small_len(&self) -> LenT {
        self.len.expand()
    }

    fn len(&self) -> usize {
        self.small_len().to_usize()
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        // SAFETY: `self.ptr` and `self.len` are both valid and derived from `Box<[T]>`.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: `self.ptr` and `self.len` are both valid and derived from `Box<[T]>`.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len()) }
    }

    /// Converts the [`NonEmptyFixedArray`] to it's original [`Box<T>`].
    ///
    /// # Safety
    /// `self` must never be used again, and it is highly recommended to wrap in [`ManuallyDrop`] before calling.
    pub(crate) unsafe fn as_box(&mut self) -> Box<[T]> {
        let slice = self.as_mut_slice();

        // SAFETY: `self` has been derived from `Box<[T]>`
        unsafe { Box::from_raw(slice) }
    }
}

impl<T, LenT: ValidLength> TryFrom<Box<[T]>> for NonEmptyFixedArray<T, LenT> {
    type Error = Option<InvalidLength<T>>;
    fn try_from(boxed_array: Box<[T]>) -> Result<Self, Self::Error> {
        let Some((len, boxed_array)) = LenT::from_usize(boxed_array)? else {
            return Err(None);
        };

        let array_ptr = Box::into_raw(boxed_array).cast::<T>();
        Ok(NonEmptyFixedArray {
            ptr: NonNull::new(array_ptr).expect("Box ptr != nullptr"),
            len,
        })
    }
}

impl<T, LenT: ValidLength> From<NonEmptyFixedArray<T, LenT>> for Box<[T]> {
    fn from(value: NonEmptyFixedArray<T, LenT>) -> Self {
        let mut value = ManuallyDrop::new(value);
        unsafe {
            // SAFETY: We don't use value again, and it is ManuallyDrop.
            value.as_box()
        }
    }
}

impl<T: Clone, LenT: ValidLength> Clone for NonEmptyFixedArray<T, LenT> {
    fn clone(&self) -> Self {
        Box::<[T]>::from(self.as_slice())
            .try_into()
            .unwrap_or_else(|_| panic!("Length of array can't change when cloning"))
    }
}

impl<T, LenT: ValidLength> Drop for NonEmptyFixedArray<T, LenT> {
    fn drop(&mut self) {
        // SAFETY: We never use `self` again, and we are in the drop impl.
        unsafe { self.as_box() };
    }
}

unsafe impl<T: Send, LenT: ValidLength> Send for NonEmptyFixedArray<T, LenT> {}
unsafe impl<T: Sync, LenT: ValidLength> Sync for NonEmptyFixedArray<T, LenT> {}
