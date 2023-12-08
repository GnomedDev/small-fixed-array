use std::{fmt::Debug, hash::Hash};

use crate::{
    length::{SmallLen, ValidLength},
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

    /// Returns the length of the [`FixedArray`].
    #[must_use]
    pub fn len(&self) -> LenT {
        self.0
            .as_ref()
            .map(NonEmptyFixedArray::small_len)
            .unwrap_or_default()
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

impl<T> std::ops::Index<u32> for FixedArray<T, u32> {
    type Output = T;
    fn index(&self, index: u32) -> &Self::Output {
        let inner: &[T] = self;
        let index: usize = index
            .try_into()
            .expect("we are indexing with a u32, and storing len as a u32");

        &inner[index]
    }
}

impl<T> std::ops::IndexMut<u32> for FixedArray<T, u32> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let inner: &mut [T] = self;
        let index: usize = index
            .try_into()
            .expect("we are indexing with a u32, and storing len as a u32");

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
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a FixedArray<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
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

impl<T, LenT: ValidLength> From<Box<[T]>> for FixedArray<T, LenT> {
    fn from(boxed_array: Box<[T]>) -> Self {
        Self((!boxed_array.is_empty()).then(|| NonEmptyFixedArray::from(boxed_array)))
    }
}

impl<T, LenT: ValidLength> From<Vec<T>> for FixedArray<T, LenT> {
    fn from(value: Vec<T>) -> Self {
        Self::from(value.into_boxed_slice())
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
        Box::<[T]>::deserialize(deserializer).map(Self::from)
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
