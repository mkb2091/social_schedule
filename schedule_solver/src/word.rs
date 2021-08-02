use core::ops::*;

pub trait Word:
    Sized
    + BitAnd<Self, Output = Self>
    + BitAndAssign<Self>
    + BitOr<Self, Output = Self>
    + BitOrAssign<Self>
    + BitXor<Self, Output = Self>
    + BitXorAssign<Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Not<Output = Self>
    + Copy
    + Clone
    + std::fmt::Debug
    + std::fmt::Binary
    + Eq
{
    const SIZE: usize = 8 * std::mem::size_of::<Self>();
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    fn count_ones(self) -> u32;
    fn count_zeros(self) -> u32;
    fn leading_ones(self) -> u32;
    fn leading_zeros(self) -> u32;
    fn trailing_ones(self) -> u32;
    fn trailing_zeros(self) -> u32;
}

macro_rules! derive_word {
    ($x: ty) => {
        impl Word for $x {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const MAX: Self = Self::MAX;
            #[inline(always)]
            fn count_ones(self) -> u32 {
                self.count_ones()
            }
            #[inline(always)]
            fn count_zeros(self) -> u32 {
                self.count_zeros()
            }
            #[inline(always)]
            fn leading_ones(self) -> u32 {
                self.leading_ones()
            }
            #[inline(always)]
            fn leading_zeros(self) -> u32 {
                self.leading_zeros()
            }
            #[inline(always)]
            fn trailing_ones(self) -> u32 {
                self.trailing_ones()
            }
            #[inline(always)]
            fn trailing_zeros(self) -> u32 {
                self.trailing_zeros()
            }
        }
    };
}

derive_word!(u8);
derive_word!(u16);
derive_word!(u32);
derive_word!(u64);
derive_word!(u128);
derive_word!(usize);
derive_word!(i8);
derive_word!(i16);
derive_word!(i32);
derive_word!(i64);
derive_word!(i128);
derive_word!(isize);
