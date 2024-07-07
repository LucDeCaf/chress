use crate::build::magics::{
    MagicEntry, BISHOP_MAGICS, BISHOP_TABLE_SIZE, ROOK_MAGICS, ROOK_TABLE_SIZE,
};

use crate::board::{bitboard::Bitboard, square::Square};

const ROOK_MOVE_OFFSETS: [i8; 4] = [1, 8, -1, -8];
const BISHOP_MOVE_OFFSETS: [i8; 4] = [7, 9, -7, -9];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slider {
    Rook,
    Bishop,
}

impl Slider {
    fn moves(&self, square: Square, blockers: Bitboard) -> Bitboard {
        let offsets = match self {
            Slider::Rook => ROOK_MOVE_OFFSETS,
            Slider::Bishop => BISHOP_MOVE_OFFSETS,
        };

        let mut moves = Bitboard::EMPTY;

        for offset in offsets {
            let mut target = square as i8;
            let target_square = Square::try_from(target as usize).unwrap();

            let mut prev_rank = target_square.rank();
            let mut prev_file = target_square.file();

            loop {
                target += offset;

                // Prevent negative squares
                if target < 0 {
                    break;
                }

                // Prevent squares >= 64
                let Ok(target_square) = Square::try_from(target as usize) else {
                    break;
                };

                // Prevent wrapping
                if target_square.rank().abs_diff(prev_rank) > 1
                    || target_square.file().abs_diff(prev_file) > 1
                {
                    break;
                }

                moves.0 |= 1 << target;

                prev_rank = target_square.rank();
                prev_file = target_square.file();

                // Break when finding piece, but only after adding blocker
                if target_square.bitboard() & blockers != Bitboard::EMPTY {
                    break;
                }
            }
        }

        moves
    }
}

pub fn magic_index(entry: &MagicEntry, blockers: Bitboard) -> usize {
    let blockers = blockers.0 & entry.mask;
    let hash = blockers.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset as usize + index
}

fn make_table(table_size: usize, slider: Slider, magics: &[MagicEntry; 64]) -> Vec<Bitboard> {
    let mut table = vec![Bitboard::EMPTY; table_size];

    for square in Square::ALL {
        let i = square as usize;

        let magic_entry = &magics[i];
        let mask = Bitboard(magic_entry.mask);

        for blockers in mask.subsets() {
            let moves = slider.moves(square, blockers);
            table[magic_index(magic_entry, blockers)] = moves;
        }
    }
    table
}

pub fn create_rook_table() -> Vec<Bitboard> {
    make_table(ROOK_TABLE_SIZE, Slider::Rook, ROOK_MAGICS)
}

pub fn create_bishop_table() -> Vec<Bitboard> {
    make_table(BISHOP_TABLE_SIZE, Slider::Bishop, BISHOP_MAGICS)
}
