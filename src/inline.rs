use nonmax::NonMaxU8;

use crate::ValidLength;

#[cfg(feature = "typesize")]
use typesize::TypeSize;

#[cfg(not(feature = "typesize"))]
pub(crate) trait TypeSize {}
#[cfg(not(feature = "typesize"))]
impl<T> TypeSize for T {}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone)]
pub(crate) struct InlineString<StrRepr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize> {
    arr: StrRepr,
    len: NonMaxU8,
}

impl<StrRepr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize> InlineString<StrRepr> {
    pub fn from_str(val: &str) -> Self {
        let len = val.len().try_into().ok();
        let len = len
            .and_then(NonMaxU8::new)
            .expect("must be less than 254 bytes");

        let mut arr = StrRepr::default();
        arr.as_mut()[..val.len()].copy_from_slice(val.as_bytes());

        Self { arr, len }
    }

    pub fn len(&self) -> u32 {
        self.len.get().into()
    }

    pub fn as_str(&self) -> &str {
        let len: usize = self.len().to_usize();
        let bytes = &self.arr.as_ref()[..len];

        // SAFETY: Accessing only initialised UTF8 bytes based on the length.
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        let len: usize = self.len().to_usize();
        let bytes = &mut self.arr.as_mut()[..len];

        // SAFETY: Accessing only initialised UTF8 bytes based on the length.
        unsafe { std::str::from_utf8_unchecked_mut(bytes) }
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
        assert_eq!(original, inline.as_str());
    }

    fn check_roundtrip_repr<Repr: Copy + AsRef<[u8]> + AsMut<[u8]> + Default + TypeSize>() {
        for i in 0..=std::mem::size_of::<Repr>() {
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
    #[should_panic(expected = "range end index 9 out of range for slice of length 8")]
    fn check_overflow() {
        check_roundtrip::<[u8; 8]>("012345678");
    }
}
