use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    impl Sealed for u32 {}
}

pub trait NonZero<T: sealed::Sealed + Copy>: Copy {
    fn expand(self) -> T;
}

impl NonZero<u8> for NonZeroU8 {
    fn expand(self) -> u8 {
        self.get()
    }
}

impl NonZero<u16> for NonZeroU16 {
    fn expand(self) -> u16 {
        self.get()
    }
}

impl NonZero<u32> for NonZeroU32 {
    fn expand(self) -> u32 {
        self.get()
    }
}

pub trait ValidLength: sealed::Sealed + Sized + Default + Copy {
    type NonZero: NonZero<Self>;

    fn from_usize(val: usize) -> Option<Self::NonZero>;
    fn to_usize(self) -> usize;
}

impl ValidLength for u8 {
    type NonZero = NonZeroU8;

    fn from_usize(val: usize) -> Option<Self::NonZero> {
        val.try_into().ok().and_then(Self::NonZero::new)
    }

    fn to_usize(self) -> usize {
        self.into()
    }
}

impl ValidLength for u16 {
    type NonZero = NonZeroU16;

    fn from_usize(val: usize) -> Option<Self::NonZero> {
        val.try_into().ok().and_then(Self::NonZero::new)
    }

    fn to_usize(self) -> usize {
        self.into()
    }
}

#[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
impl ValidLength for u32 {
    type NonZero = NonZeroU32;

    fn from_usize(val: usize) -> Option<Self::NonZero> {
        val.try_into().ok().and_then(Self::NonZero::new)
    }

    fn to_usize(self) -> usize {
        self.try_into()
            .expect("u32 can fit into usize on platforms with pointer lengths of 32 and 64")
    }
}

#[cfg(target_pointer_width = "16")]
pub type SmallLen = u16;
#[cfg(not(target_pointer_width = "16"))]
pub type SmallLen = u32;
