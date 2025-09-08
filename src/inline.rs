use core::mem::size_of;

use crate::ValidLength;

#[cfg(feature = "typesize")]
use typesize::TypeSize;

#[cfg(not(feature = "typesize"))]
pub(crate) trait TypeSize {}
#[cfg(not(feature = "typesize"))]
impl<T> TypeSize for T {}

#[must_use]
pub(crate) const fn get_heap_threshold<LenT>() -> usize {
    core::mem::size_of::<usize>() + core::mem::size_of::<LenT>()
}

#[cfg(not(feature = "nightly"))]
fn find_term_index(haystack: [u8; 16], term: u8, fallback: u8) -> u8 {
    let mut term_position = fallback;

    // Avoid enumerate to keep the index as a u8
    for (pos, byte) in (0..16).zip(haystack) {
        if byte == term {
            // Do not break, it reduces performance a ton due to branching.
            term_position = pos;
        }
    }

    term_position
}

#[cfg(feature = "nightly")]
fn find_term_index(haystack: [u8; 16], term: u8, fallback: u8) -> u8 {
    use core::simd::prelude::*;

    // Make simd array of [term; 16]
    let term_arr = u8x16::splat(term);
    // Convert haystack into simd array
    let elements = u8x16::from_array(haystack);
    // Compare each element of the simd array, converting back to a scalar bitmask.
    let scalar_mask = term_arr.simd_eq(elements).to_bitmask();

    if scalar_mask == 0 {
        // If the mask is 0, the terminator was not included, so return fallback.
        fallback
    } else {
        // The mask has the terminator as the last character with a bit set, so use trailing zeros.
        u8::try_from(scalar_mask.trailing_zeros()).unwrap()
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone)]
pub(crate) struct InlineString<StrRepr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize> {
    arr: StrRepr,
}

impl<StrRepr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize> InlineString<StrRepr> {
    const TERMINATOR: u8 = 0xFF;

    fn max_len() -> usize {
        StrRepr::default().as_ref().len()
    }

    #[inline]
    fn from_len_and_write(len: usize, write: impl FnOnce(&mut [u8])) -> Option<Self> {
        let mut arr = StrRepr::default();
        if len > size_of::<Self>() {
            return None;
        }

        write(arr.as_mut());

        if len != Self::max_len() {
            // 0xFF terminate the string, to gain an extra inline character
            arr.as_mut()[len] = Self::TERMINATOR;
        }

        Some(Self { arr })
    }

    pub fn from_str(val: &str) -> Option<Self> {
        Self::from_len_and_write(val.len(), |arr| {
            arr[..val.len()].copy_from_slice(val.as_bytes());
        })
    }

    pub fn from_char(val: char) -> Option<Self> {
        Self::from_len_and_write(val.len_utf8(), |arr| {
            val.encode_utf8(arr);
        })
    }

    pub fn len(&self) -> u8 {
        // Copy to a temporary, 16 byte array to allow for SIMD impl.
        let mut buf = [0_u8; 16];
        buf[..Self::max_len()].copy_from_slice(self.arr.as_ref());

        // This call is different depending on nightly or not.
        find_term_index(buf, Self::TERMINATOR, Self::max_len().try_into().unwrap())
    }

    pub fn as_str(&self) -> &str {
        let len: usize = self.len().to_usize();
        let bytes = &self.arr.as_ref()[..len];

        // SAFETY: Accessing only initialised UTF8 bytes based on the length.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }
}

impl<Repr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize> Copy for InlineString<Repr> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_roundtrip<Repr>(original: &str)
    where
        Repr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize,
    {
        let inline = InlineString::<Repr>::from_str(original);
        assert_eq!(original, inline.expect("should not overflow").as_str());
    }

    fn check_roundtrip_repr<Repr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize>() {
        for i in 0..=core::mem::size_of::<Repr>() {
            let original = "a".repeat(i);
            check_roundtrip::<Repr>(&original);
        }
    }

    #[test]
    fn roundtrip_tests() {
        check_roundtrip_repr::<<u8 as ValidLength>::InlineStrRepr>();
        check_roundtrip_repr::<<u16 as ValidLength>::InlineStrRepr>();
        check_roundtrip_repr::<<u32 as ValidLength>::InlineStrRepr>();
    }

    #[test]
    #[should_panic(expected = "should not overflow")]
    fn check_overflow() {
        check_roundtrip::<[u8; 8]>("012345678");
    }
}
