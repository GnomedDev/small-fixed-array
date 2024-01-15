use crate::{FixedArray, FixedString, ValidLength};
mod sealed {
    pub trait Sealed {}

    impl Sealed for String {}
    impl<T> Sealed for Vec<T> {}
}

/// A sealed helper trait for calling [`FixedArray<T>::from_vec_trunc`] or [`FixedString::from_string_trunc`].
///
/// Both of these functions may truncate the input in order to fit it into the provided [`ValidLength`],
/// therefore this trait must be imported in order to make possible truncation made obvious in user code.
pub trait TruncatingInto<T>: sealed::Sealed {
    fn trunc_into(self) -> T;
}

impl<LenT: ValidLength> TruncatingInto<FixedString<LenT>> for String {
    fn trunc_into(self) -> FixedString<LenT> {
        FixedString::from_string_trunc(self)
    }
}

impl<T, LenT: ValidLength> TruncatingInto<FixedArray<T, LenT>> for Vec<T> {
    fn trunc_into(self) -> FixedArray<T, LenT> {
        FixedArray::from_vec_trunc(self)
    }
}
