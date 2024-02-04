use core::ptr::NonNull;

use crate::ValidLength;

#[repr(packed)]
#[derive(Clone, Copy)]
pub(crate) struct StaticStr<LenT: ValidLength> {
    ptr: NonNull<u8>,
    len: LenT,
}

impl<LenT: ValidLength> StaticStr<LenT> {
    /// # Panics
    /// Panics if the string passed requires truncation.
    pub fn from_static_str(src: &'static str) -> Self {
        let ptr = NonNull::new(src.as_ptr().cast_mut()).expect("str::as_ptr should never be null");
        let len = LenT::from_usize(src.len()).unwrap();

        Self { ptr, len }
    }

    pub fn as_str(&self) -> &'static str {
        unsafe {
            let slice = core::slice::from_raw_parts(self.ptr.as_ptr(), self.len.to_usize());
            core::str::from_utf8_unchecked(slice)
        }
    }

    pub fn len(&self) -> LenT {
        self.len
    }
}

unsafe impl<LenT: ValidLength> Send for StaticStr<LenT> {}
unsafe impl<LenT: ValidLength> Sync for StaticStr<LenT> {}

#[cfg(feature = "typesize")]
impl<LenT: ValidLength> typesize::TypeSize for StaticStr<LenT> {}
