pub trait Bit {
    fn mask(nbits: usize) -> Self;
    fn bit(self, pos: usize) -> bool;
    fn bits(self, hi: usize, lo: usize) -> Self;
    fn set_bit(self, pos: usize, val: bool) -> Self;
    fn set_bits(self, hi: usize, lo: usize, val: Self) -> Self;
}

macro_rules! bitindex_num_impl {
    ($($t:ty),*) => {$(
        impl Bit for $t {
            #[inline]
            fn mask(nbits: usize) -> Self {
                if nbits == ::core::mem::size_of::<Self>() * 8 {
                    !(0 as Self)
                } else {
                    ((1 as Self) << nbits) - 1
                }
            }

            #[inline]
            fn bit(self, pos: usize) -> bool {
                (self >> pos) & 1 != 0
            }

            #[inline]
            fn bits(self, hi: usize, lo: usize) -> Self {
                (self >> lo) & Self::mask(hi - lo + 1)
            }

            #[inline]
            fn set_bit(self, pos: usize, val: bool) -> Self {
                (self & !((1 as Self) << pos)) | ((val as Self) << pos)
            }

            #[inline]
            fn set_bits(self, hi: usize, lo: usize, val: Self) -> Self {
                (self & !(Self::mask(hi - lo + 1) << lo)) | (val << lo)
            }
        }
    )*}
}

bitindex_num_impl!(u8, u16, u32, u64, usize);
