macro_rules! bitfield {
    ($field:ident: $ty:ty; $($get_name:ident, $set_name:ident: $hi:expr, $lo:expr);+ $(;)?) => {
        $(
            fn $get_name(&self) -> u64 {
                use $crate::bit::Bit;
                self.$field.bits($hi, $lo)
            }

            fn $set_name(&mut self, val: u64) {
                use $crate::bit::Bit;
                self.$field = self.$field.set_bits($hi, $lo, val);
            }
        )+
    };
}
