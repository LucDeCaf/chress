#[rustfmt::skip]
pub mod masks {
    use super::*;

    pub const BLACK_KINGSIDE : Flags = Flags(0b0000_0001);
    pub const BLACK_QUEENSIDE: Flags = Flags(0b0000_0010);
    pub const WHITE_KINGSIDE : Flags = Flags(0b0000_0100);
    pub const WHITE_QUEENSIDE: Flags = Flags(0b0000_1000);
    pub const EP_FILE        : Flags = Flags(0b0111_0000);
    pub const EP_IS_VALID    : Flags = Flags(0b1000_0000);
}

use std::{fmt::Display, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not}};

use masks::*;

use super::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags(pub u8);

impl Flags {
    pub fn new(flags: u8) -> Self {
        Self(flags)
    }

    pub fn white_kingside(&self) -> bool {
        (self.0 & WHITE_KINGSIDE.0) != 0
    }

    pub fn black_kingside(&self) -> bool {
        (self.0 & BLACK_KINGSIDE.0) != 0
    }

    pub fn white_queenside(&self) -> bool {
        (self.0 & WHITE_QUEENSIDE.0) != 0
    }

    pub fn black_queenside(&self) -> bool {
        (self.0 & BLACK_QUEENSIDE.0) != 0
    }

    pub fn kingside(&self, color: Color) -> bool {
        (self.0 >> (color as u8 * 2)) & BLACK_KINGSIDE.0 != 0
    }

    pub fn queenside(&self, color: Color) -> bool {
        (self.0 >> (color as u8 * 2)) & BLACK_QUEENSIDE.0 != 0
    }

    pub fn en_passant_file(&self) -> Option<u8> {
        if self.en_passant_valid() {
            Some((*self & EP_FILE).0 >> 4)
        } else {
            None
        }
    }

    /// Returns the en passant file, regardless of whether en passant is valid or not.
    pub fn en_passant_file_unchecked(&self) -> u8 {
        (self.0 & EP_FILE.0) >> 4
    }

    /// Determines whether en passant is valid or not.
    ///
    /// This does NOT determine whether it is legal/pseudolegal,
    /// only whether or not the en passant mask should exist or not.
    pub fn en_passant_valid(&self) -> bool {
        (self.0 & EP_IS_VALID.0) != 0
    }
}

impl Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08b}", self.0)
    }
}

macro_rules! impl_bit_ops {
    ($op:ident, $fn:ident, $ex:tt) => {
        impl $op for Flags {
            type Output = Flags;
            fn $fn(self, rhs: Self) -> Self::Output {
                Flags(self.0 $ex rhs.0)
            }
        }
    };
}

macro_rules! impl_bit_ops_assign {
    ($op:ident, $fn:ident, $ex:tt) => {
        impl $op for Flags {
            fn $fn(&mut self, rhs: Self) {
                self.0 $ex rhs.0;
            }
        }
    };
}

impl_bit_ops!(BitAnd, bitand, &);
impl_bit_ops!(BitOr, bitor, |);
impl_bit_ops!(BitXor, bitxor, ^);

impl_bit_ops_assign!(BitAndAssign, bitand_assign, &=);
impl_bit_ops_assign!(BitOrAssign, bitor_assign, |=);
impl_bit_ops_assign!(BitXorAssign, bitxor_assign, ^=);

impl Not for Flags {
    type Output = Flags;
    fn not(self) -> Self::Output {
        Flags(!self.0)
    }
}
