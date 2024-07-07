use std::fmt::Display;

use super::bitboard::Bitboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    pub const ALL: [Square; 64] = [
        Square::A1,
        Square::B1,
        Square::C1,
        Square::D1,
        Square::E1,
        Square::F1,
        Square::G1,
        Square::H1,
        Square::A2,
        Square::B2,
        Square::C2,
        Square::D2,
        Square::E2,
        Square::F2,
        Square::G2,
        Square::H2,
        Square::A3,
        Square::B3,
        Square::C3,
        Square::D3,
        Square::E3,
        Square::F3,
        Square::G3,
        Square::H3,
        Square::A4,
        Square::B4,
        Square::C4,
        Square::D4,
        Square::E4,
        Square::F4,
        Square::G4,
        Square::H4,
        Square::A5,
        Square::B5,
        Square::C5,
        Square::D5,
        Square::E5,
        Square::F5,
        Square::G5,
        Square::H5,
        Square::A6,
        Square::B6,
        Square::C6,
        Square::D6,
        Square::E6,
        Square::F6,
        Square::G6,
        Square::H6,
        Square::A7,
        Square::B7,
        Square::C7,
        Square::D7,
        Square::E7,
        Square::F7,
        Square::G7,
        Square::H7,
        Square::A8,
        Square::B8,
        Square::C8,
        Square::D8,
        Square::E8,
        Square::F8,
        Square::G8,
        Square::H8,
    ];

    pub const fn bitboard(&self) -> Bitboard {
        Bitboard(1 << *self as u8)
    }

    pub const fn rank(&self) -> u8 {
        *self as u8 / 8
    }

    pub const fn file(&self) -> u8 {
        *self as u8 % 8
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = (self.rank() + b'1') as char;
        let file = (self.file() + b'a') as char;

        write!(f, "{file}{rank}")
    }
}

#[derive(Debug)]
pub enum ParseSquareError {
    OutOfRange,
    BadValue,
}

impl TryFrom<usize> for Square {
    type Error = ParseSquareError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::ALL
            .get(value)
            .cloned()
            .ok_or(ParseSquareError::OutOfRange)
    }
}

impl TryFrom<&str> for Square {
    type Error = ParseSquareError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_ascii_lowercase().chars().collect::<Vec<char>>();
        let rank = value[1] as usize - '1' as usize;
        let file = value[0] as usize - 'a' as usize;

        Square::try_from(rank * 8 + file)
    }
}

impl TryFrom<Bitboard> for Square {
    type Error = ParseSquareError;

    fn try_from(value: Bitboard) -> Result<Self, Self::Error> {
        if value.0.count_ones() != 1 {
            return Err(ParseSquareError::BadValue);
        }

        let square_index = value.0.trailing_zeros() as usize;

        Self::try_from(square_index)
    }
}
