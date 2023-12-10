use std::{fmt::Debug, hash::Hash};

use crate::{
    length::{InvalidLength, SmallLen, ValidLength},
    logging::error,
    non_empty_array::NonEmptyFixedArray,
};

/// A fixed size array with length provided at creation denoted in a [`ValidLength`], by default [`u32`].
///
/// See module level documentation for more information.
#[derive(Clone)]
pub struct FixedArray<T, LenT: ValidLength = SmallLen>(Option<NonEmptyFixedArray<T, LenT>>);

impl<T, LenT: ValidLength> FixedArray<T, LenT> {
    /// Alias to [`FixedArray::empty`].
    #[must_use]
    pub fn new() -> Self {
        Self::empty()
    }

    /// Creates a new, empty [`FixedArray`] that cannot be pushed to.
    #[must_use]
    pub fn empty() -> Self {
        Self(None)
    }

    pub(crate) fn small_len(&self) -> LenT {
        self.0
            .as_ref()
            .map(NonEmptyFixedArray::small_len)
            .unwrap_or_default()
    }

    /// Returns the length of the [`FixedArray`].
    #[must_use]
    pub fn len(&self) -> u32 {
        self.small_len().to_u32()
    }

    /// Returns if the length is equal to 0.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
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

    /// Converts `&`[`FixedArray<T>`] to `&[T]`, this conversion can be performed by [`std::ops::Deref`].
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Converts `&mut `[`FixedArray<T>`] to `&mut [T]`, this conversion can be performed by [`std::ops::DerefMut`].
    #[must_use]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, LenT: ValidLength> Default for FixedArray<T, LenT> {
    /// Creates a new, empty [`FixedArray`] that cannot be pushed to.
    fn default() -> Self {
        Self::empty()
    }
}

impl<T, LenT: ValidLength> std::ops::Deref for FixedArray<T, LenT> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .map(NonEmptyFixedArray::as_slice)
            .unwrap_or_default()
    }
}

impl<T, LenT: ValidLength> std::ops::DerefMut for FixedArray<T, LenT> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .map(NonEmptyFixedArray::as_mut_slice)
            .unwrap_or_default()
    }
}

impl<T, LenT: ValidLength> std::ops::Index<usize> for FixedArray<T, LenT> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let inner: &[T] = self;
        &inner[index]
    }
}

impl<T, LenT: ValidLength> std::ops::IndexMut<usize> for FixedArray<T, LenT> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let inner: &mut [T] = self;
        &mut inner[index]
    }
}

impl<T: Hash, LenT: ValidLength> Hash for FixedArray<T, LenT> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T: PartialEq, LenT: ValidLength> PartialEq for FixedArray<T, LenT> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T: Eq, LenT: ValidLength> Eq for FixedArray<T, LenT> {}

impl<T: Debug, LenT: ValidLength> Debug for FixedArray<T, LenT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl<T, LenT: ValidLength> std::iter::FromIterator<T> for FixedArray<T, LenT> {
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        Vec::from_iter(iter).into()
    }
}

impl<T, LenT: ValidLength> From<FixedArray<T, LenT>> for Box<[T]> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        value.0.map(Box::from).unwrap_or_default()
    }
}

impl<T, LenT: ValidLength> From<FixedArray<T, LenT>> for Vec<T> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        value.into_boxed_slice().into_vec()
    }
}

impl<T, LenT: ValidLength> TryFrom<Box<[T]>> for FixedArray<T, LenT> {
    type Error = InvalidLength;
    fn try_from(boxed_array: Box<[T]>) -> Result<Self, Self::Error> {
        match NonEmptyFixedArray::try_from(boxed_array) {
            Ok(arr) => Ok(Self(Some(arr))),
            Err(None) => Ok(Self(None)),
            Err(Some(err)) => Err(err),
        }
    }
}

impl<T, LenT: ValidLength> From<Vec<T>> for FixedArray<T, LenT> {
    fn from(mut value: Vec<T>) -> Self {
        if value.len() >= LenT::MAX {
            let max_len = LenT::MAX;
            error!("Truncated Vec<T> to fit into max len {max_len}");
            value.truncate(max_len);
        }

        value
            .into_boxed_slice()
            .try_into()
            .expect("array can fit in size as just truncated")
    }
}

impl<T, LenT: ValidLength> AsRef<[T]> for FixedArray<T, LenT> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, LenT: ValidLength> From<FixedArray<T, LenT>> for std::sync::Arc<[T]> {
    fn from(value: FixedArray<T, LenT>) -> Self {
        std::sync::Arc::from(value.into_boxed_slice())
    }
}

#[cfg(feature = "serde")]
impl<'de, T, LenT> serde::Deserialize<'de> for FixedArray<T, LenT>
where
    T: serde::Deserialize<'de>,
    LenT: ValidLength,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<T>::deserialize(deserializer).map(Self::from)
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
