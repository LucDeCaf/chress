use std::{error::Error, fmt::Display};

use super::r#move::Move;

#[derive(Debug, Clone, Copy)]
pub struct ParsePieceCharError;

impl Display for ParsePieceCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad piece char")
    }
}

// test

impl Error for ParsePieceCharError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[repr(u8)]
pub enum Piece {
    // Order like this for branchless promotions
    Knight,
    Bishop,
    Rook,
    Queen,
    Pawn,
    King,
}

impl Piece {
    pub const ALL: [Piece; 6] = [
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::Pawn,
        Piece::King,
    ];
    pub const CHARS: [char; 6] = ['n', 'b', 'r', 'q', 'p', 'k'];

    pub const PROMOTION_MASKS: [u16; 4] = [
        0b0000_0000_0000_0001,
        0b0000_0000_0000_0010,
        0b0000_0000_0000_0100,
        0b0000_0000_0000_1000,
    ];

    pub const fn promotion_mask(&self) -> u16 {
        (1 << *self as u16) & Move::PROMOTION_MASK
    }
}

const OFFSET: usize = 'A' as usize;

const LOOKUP: [Option<Piece>; 58] = {
    let mut table = [None; 58];
    table['N' as usize - OFFSET] = Some(Piece::Knight);
    table['n' as usize - OFFSET] = Some(Piece::Knight);
    table['B' as usize - OFFSET] = Some(Piece::Bishop);
    table['b' as usize - OFFSET] = Some(Piece::Bishop);
    table['R' as usize - OFFSET] = Some(Piece::Rook);
    table['r' as usize - OFFSET] = Some(Piece::Rook);
    table['Q' as usize - OFFSET] = Some(Piece::Queen);
    table['q' as usize - OFFSET] = Some(Piece::Queen);
    table['P' as usize - OFFSET] = Some(Piece::Pawn);
    table['p' as usize - OFFSET] = Some(Piece::Pawn);
    table['K' as usize - OFFSET] = Some(Piece::King);
    table['k' as usize - OFFSET] = Some(Piece::King);
    table
};

impl TryFrom<char> for Piece {
    type Error = ParsePieceCharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        LOOKUP
            .get(value as usize - OFFSET)
            .cloned()
            .flatten()
            .ok_or(ParsePieceCharError)
    }
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        Piece::CHARS[value as usize]
    }
}
