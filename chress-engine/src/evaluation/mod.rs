use chress::{board::Board, color::Color, piece::Piece};

use crate::r#static::PIECE_SCORES;

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;

    for piece in Piece::ALL {
        for color in Color::ALL {
            let bb = board.bitboard(piece, color);

            score +=
                bb.0.count_ones() as i32 * PIECE_SCORES[piece as usize] * color.direction() as i32;
        }
    }

    score
}
