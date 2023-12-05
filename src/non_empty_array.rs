use std::{mem::ManuallyDrop, num::NonZeroU32, ptr::NonNull};

#[repr(packed)]
pub(crate) struct NonEmptyFixedArray<T> {
    ptr: NonNull<T>,
    len: NonZeroU32,
}

impl<T> NonEmptyFixedArray<T> {
    pub(crate) fn small_len(&self) -> u32 {
        self.len.get()
    }

    fn len(&self) -> usize {
        self.small_len() as usize
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        // SAFETY: `self.ptr` and `self.len` are both valid and derived from `Box<[T]>`.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: `self.ptr` and `self.len` are both valid and derived from `Box<[T]>`.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len()) }
    }

    /// Converts the NonEmptyFixedArray to it's original [`Box<T>`].
    ///
    /// # Safety
    /// `self` must never be used again, and it is highly recommended to wrap in ManuallyDrop before calling.
    pub(crate) unsafe fn as_box(&mut self) -> Box<[T]> {
        let slice = self.as_mut_slice();

        // SAFETY: `self` has been derived from `Box<[T]>`
        unsafe { Box::from_raw(slice as *mut [T]) }
    }
}

impl<T> From<Box<[T]>> for NonEmptyFixedArray<T> {
    fn from(boxed_array: Box<[T]>) -> Self {
        let len = NonZeroU32::new(boxed_array.len().try_into().unwrap()).unwrap();
        let array_ptr = Box::into_raw(boxed_array) as *mut T;

        NonEmptyFixedArray {
            ptr: NonNull::new(array_ptr).expect("Box ptr != nullptr"),
            len,
        }
    }
}

impl<T> From<NonEmptyFixedArray<T>> for Box<[T]> {
    fn from(value: NonEmptyFixedArray<T>) -> Self {
        let mut value = ManuallyDrop::new(value);
        unsafe {
            // SAFETY: We don't use value again, and it is ManuallyDrop.
            value.as_box()
        }
    }
}

impl<T: Clone> Clone for NonEmptyFixedArray<T> {
    fn clone(&self) -> Self {
        self.as_slice().to_vec().into_boxed_slice().into()
    }
}

impl<T> Drop for NonEmptyFixedArray<T> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: We never use `self` again, and we are in the drop impl.
            self.as_box();
        }
    }
}

unsafe impl<T: Send> Send for NonEmptyFixedArray<T> {}
unsafe impl<T: Sync> Sync for NonEmptyFixedArray<T> {}
