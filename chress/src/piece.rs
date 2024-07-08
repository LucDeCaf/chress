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

#[cfg(test)]
mod piece_tests {
    extern crate test;

    use super::*;

    use rand::{thread_rng, Rng};
    use test::Bencher;

    // 27.50 ± 0.5-1.00
    #[bench]
    fn char_to_piece_branched(b: &mut Bencher) {
        let mut rng = thread_rng();
        const CHARS: [char; 12] = ['n', 'b', 'r', 'k', 'q', 'p', 'N', 'B', 'R', 'K', 'Q', 'P'];

        b.iter(|| {
            let value = CHARS[rng.gen_range(0..12)];

            match value {
                'n' | 'N' => Ok(Piece::Knight),
                'b' | 'B' => Ok(Piece::Bishop),
                'r' | 'R' => Ok(Piece::Rook),
                'q' | 'Q' => Ok(Piece::Queen),
                'p' | 'P' => Ok(Piece::Pawn),
                'k' | 'K' => Ok(Piece::King),
                _ => Err(ParsePieceCharError),
            }
        })
    }

    // 14.8 ± 0.5
    #[bench]
    fn char_to_piece_minimally_branched_table(b: &mut Bencher) {
        let mut rng = thread_rng();

        const CHARS: [char; 12] = ['n', 'b', 'r', 'k', 'q', 'p', 'N', 'B', 'R', 'K', 'Q', 'P'];

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

        b.iter(|| {
            let value = CHARS[rng.gen_range(0..12)];

            LOOKUP
                .get(value as usize - OFFSET)
                .cloned()
                .flatten()
                .ok_or(ParsePieceCharError)
        })
    }

    // 14.8 ± 0.6
    #[bench]
    fn char_to_piece_minimally_branched_result_table(b: &mut Bencher) {
        let mut rng = thread_rng();

        const CHARS: [char; 12] = ['n', 'b', 'r', 'k', 'q', 'p', 'N', 'B', 'R', 'K', 'Q', 'P'];

        const OFFSET: usize = 'A' as usize;

        const LOOKUP: [Result<Piece, ParsePieceCharError>; 58] = {
            let mut table = [Err(ParsePieceCharError); 58];
            table['N' as usize - OFFSET] = Ok(Piece::Knight);
            table['n' as usize - OFFSET] = Ok(Piece::Knight);
            table['B' as usize - OFFSET] = Ok(Piece::Bishop);
            table['b' as usize - OFFSET] = Ok(Piece::Bishop);
            table['R' as usize - OFFSET] = Ok(Piece::Rook);
            table['r' as usize - OFFSET] = Ok(Piece::Rook);
            table['Q' as usize - OFFSET] = Ok(Piece::Queen);
            table['q' as usize - OFFSET] = Ok(Piece::Queen);
            table['P' as usize - OFFSET] = Ok(Piece::Pawn);
            table['p' as usize - OFFSET] = Ok(Piece::Pawn);
            table['K' as usize - OFFSET] = Ok(Piece::King);
            table['k' as usize - OFFSET] = Ok(Piece::King);
            table
        };

        b.iter(|| {
            let value = CHARS[rng.gen_range(0..12)];

            LOOKUP.get(value as usize - OFFSET).cloned()
        })
    }

    // 14.8 ± 0.5
    // Much larger table for very minimal gains
    #[bench]
    fn char_to_piece_branchless_table(b: &mut Bencher) {
        let mut rng = thread_rng();
        const CHARS: [char; 12] = ['n', 'b', 'r', 'k', 'q', 'p', 'N', 'B', 'R', 'K', 'Q', 'P'];

        const LOOKUP: [Result<Piece, ParsePieceCharError>; char::MAX as usize] = {
            let mut table = [Err(ParsePieceCharError); char::MAX as usize];

            table['N' as usize] = Ok(Piece::Knight);
            table['n' as usize] = Ok(Piece::Knight);
            table['B' as usize] = Ok(Piece::Bishop);
            table['b' as usize] = Ok(Piece::Bishop);
            table['R' as usize] = Ok(Piece::Rook);
            table['r' as usize] = Ok(Piece::Rook);
            table['Q' as usize] = Ok(Piece::Queen);
            table['q' as usize] = Ok(Piece::Queen);
            table['P' as usize] = Ok(Piece::Pawn);
            table['p' as usize] = Ok(Piece::Pawn);
            table['K' as usize] = Ok(Piece::King);
            table['k' as usize] = Ok(Piece::King);

            table
        };

        b.iter(|| {
            let value = CHARS[rng.gen_range(0..12)];

            LOOKUP[value as usize]
        })
    }

    // ~16.10
    #[bench]
    fn piece_to_char_branched(b: &mut Bencher) {
        let mut rng = thread_rng();

        b.iter(|| {
            let value = Piece::ALL[rng.gen_range(0..6)];

            match value {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                Piece::Pawn => 'p',
                Piece::King => 'k',
            }
        })
    }

    // ~14.70, ±0.21-0.38
    #[bench]
    fn piece_to_char_branchless(b: &mut Bencher) {
        const C: [char; 6] = ['n', 'b', 'r', 'q', 'p', 'k'];
        let mut rng = thread_rng();

        b.iter(|| {
            let value = Piece::ALL[rng.gen_range(0..6)];

            C[value as usize]
        })
    }

    // ~14.70, ±0.16-16.0
    #[bench]
    fn piece_to_char_unsafe(b: &mut Bencher) {
        const C: [char; 6] = ['n', 'b', 'r', 'q', 'p', 'k'];
        let mut rng = thread_rng();

        b.iter(|| {
            let value = Piece::ALL[rng.gen_range(0..6)];

            unsafe { C.get_unchecked(value as usize) }
        })
    }
}
