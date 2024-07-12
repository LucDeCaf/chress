use core::fmt;
use std::{
    fmt::Display,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, ShlAssign,
        Shr, ShrAssign,
    },
};

use crate::board::r#move::Move;

use super::square::Square;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);
    pub const UNIVERSE: Bitboard = Bitboard(u64::MAX);

    pub fn subsets(&self) -> Vec<Bitboard> {
        let mut subsets = vec![];

        let set = self.0;
        let mut subset = 0;

        loop {
            subsets.push(Bitboard(subset));

            subset = subset.wrapping_sub(set) & set;
            if subset == 0 {
                break;
            }
        }

        subsets
    }

    /// Appends moves to a move list
    pub fn append_moves_from(&mut self, moves: &mut Vec<Move>, from: Square) {
        for _ in 0..self.0.count_ones() {
            moves.push(Move::new(from, Square::ALL[self.pop_lsb() as usize]));
        }
    }

    /// Pops the least significant bit, returning its index in the bitboard.
    pub fn pop_lsb(&mut self) -> u32 {
        let i = self.0.trailing_zeros();
        self.0 &= self.0 - 1;
        i
    }

    pub fn is_empty(&self) -> bool {
        *self == Bitboard::EMPTY
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = vec![String::with_capacity(8); 8];
        let mut i = 0;

        for square in Square::ALL {
            let line = &mut lines[i];

            line.push(if *self & square.bitboard() != Bitboard::EMPTY {
                '1'
            } else {
                '0'
            });

            if square.file() == 7 {
                i += 1;
            } else {
                line.push(' ');
            }
        }

        let mut lines = lines.into_iter().rev();

        let mut buf = String::with_capacity(64 * 2 + 7);
        buf.push_str(&lines.next().unwrap());

        for line in lines {
            buf.push('\n');
            buf.push_str(&line);
        }

        write!(f, "{}", buf)
    }
}

impl Mul<bool> for Bitboard {
    type Output = Bitboard;
    fn mul(self, rhs: bool) -> Self::Output {
        Bitboard(self.0 * rhs as u64)
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl Shl<Self> for Bitboard {
    type Output = Bitboard;
    fn shl(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 << rhs.0)
    }
}

impl Shr<Self> for Bitboard {
    type Output = Bitboard;
    fn shr(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 >> rhs.0)
    }
}

impl ShlAssign<Self> for Bitboard {
    fn shl_assign(&mut self, rhs: Self) {
        self.0 <<= rhs.0;
    }
}

impl ShrAssign<Self> for Bitboard {
    fn shr_assign(&mut self, rhs: Self) {
        self.0 >>= rhs.0;
    }
}

macro_rules! impl_bit_ops {
    ($op:ident, $fn:ident, $ex:tt) => {
        impl $op for Bitboard {
            type Output = Bitboard;
            fn $fn(self, rhs: Self) -> Self::Output {
                Bitboard(self.0 $ex rhs.0)
            }
        }
    };
}

macro_rules! impl_bit_ops_assign {
    ($op:ident, $fn:ident, $ex:tt) => {
        impl $op for Bitboard {
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

macro_rules! impl_shift {
    ($t:ty) => {
        impl Shl<$t> for Bitboard {
            type Output = Bitboard;
            fn shl(self, rhs: $t) -> Self::Output {
                Bitboard(self.0 << rhs)
            }
        }

        impl ShlAssign<$t> for Bitboard {
            fn shl_assign(&mut self, rhs: $t) {
                self.0 <<= rhs;
            }
        }

        impl Shr<$t> for Bitboard {
            type Output = Bitboard;
            fn shr(self, rhs: $t) -> Self::Output {
                Bitboard(self.0 >> rhs)
            }
        }

        impl ShrAssign<$t> for Bitboard {
            fn shr_assign(&mut self, rhs: $t) {
                self.0 >>= rhs;
            }
        }
    };
}

impl_shift!(u8);
impl_shift!(u16);
impl_shift!(u32);
impl_shift!(u64);
impl_shift!(u128);
impl_shift!(usize);
impl_shift!(i8);
impl_shift!(i16);
impl_shift!(i32);
impl_shift!(i64);
impl_shift!(i128);
impl_shift!(isize);
