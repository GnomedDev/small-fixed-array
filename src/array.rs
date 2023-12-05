use std::{fmt::Debug, hash::Hash};

use crate::non_empty_array::NonEmptyFixedArray;

/// A fixed size array with length provided at creation with length denoted in [`u32`]
#[derive(Clone)]
pub struct FixedArray<T>(Option<NonEmptyFixedArray<T>>);

impl<T> FixedArray<T> {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn empty() -> Self {
        Self(None)
    }

    pub fn len(&self) -> u32 {
        self.0
            .as_ref()
            .map(NonEmptyFixedArray::small_len)
            .unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn as_slice(&self) -> &[T] {
        self
    }

    pub fn into_vec(self) -> Vec<T> {
        self.into()
    }
}

impl<T> Default for FixedArray<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> std::ops::Deref for FixedArray<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.0
            .as_ref()
            .map(NonEmptyFixedArray::as_slice)
            .unwrap_or_default()
    }
}

impl<T> std::ops::DerefMut for FixedArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
            .as_mut()
            .map(NonEmptyFixedArray::as_mut_slice)
            .unwrap_or_default()
    }
}

impl<T> std::ops::Index<u32> for FixedArray<T> {
    type Output = T;
    fn index(&self, index: u32) -> &Self::Output {
        let inner: &[T] = self;
        &inner[index as usize]
    }
}

impl<T> std::ops::IndexMut<u32> for FixedArray<T> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let inner: &mut [T] = self;
        &mut inner[index as usize]
    }
}

impl<T: Hash> Hash for FixedArray<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}

impl<T: PartialEq> PartialEq for FixedArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T: Eq> Eq for FixedArray<T> {}

impl<T: Debug> Debug for FixedArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <[T] as Debug>::fmt(self, f)
    }
}

impl<T> IntoIterator for FixedArray<T> {
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

impl<T> std::iter::FromIterator<T> for FixedArray<T> {
    fn from_iter<Iter: IntoIterator<Item = T>>(iter: Iter) -> Self {
        Vec::from_iter(iter).into()
    }
}

impl<T> From<FixedArray<T>> for Box<[T]> {
    fn from(value: FixedArray<T>) -> Self {
        value.0.map(Box::from).unwrap_or_default()
    }
}

impl<T> From<FixedArray<T>> for Vec<T> {
    fn from(value: FixedArray<T>) -> Self {
        let boxed_array: Box<[T]> = value.into();
        boxed_array.into_vec()
    }
}

impl<T> From<Box<[T]>> for FixedArray<T> {
    fn from(boxed_array: Box<[T]>) -> Self {
        Self((!boxed_array.is_empty()).then(|| NonEmptyFixedArray::from(boxed_array)))
    }
}

impl<T> From<Vec<T>> for FixedArray<T> {
    fn from(value: Vec<T>) -> Self {
        let boxed_array = value.into_boxed_slice();
        Self::from(boxed_array)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for FixedArray<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Box::<[T]>::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for FixedArray<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "typesize")]
impl<T: typesize::TypeSize> typesize::TypeSize for FixedArray<T> {
    fn extra_size(&self) -> usize {
        self.iter().map(T::get_size).sum()
    }
}