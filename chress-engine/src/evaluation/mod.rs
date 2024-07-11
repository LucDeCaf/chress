use chress::board::{color::Color, piece::Piece, Board};

pub const PIECE_SCORES: [i32; 6] = [320, 350, 500, 900, 100, 20000];

#[rustfmt::skip]
pub const PIECE_SQUARE_TABLES: [[i32; 64]; 6] = [
        [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ],
    [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ],
    [
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0,
    ],
    [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5,  5,  5,  5,  0,-10,
         -5,  0,  5,  5,  5,  5,  0, -5,
          0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  0,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20,
    ],
    [
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -10,-20,-20,-20,-20,-20,-20,-10,
         20, 20,  0,  0,  0,  0, 20, 20,
         20, 30, 10,  0,  0, 10, 30, 20,
    ],
    [
        0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
        5,  5, 10, 25, 25, 10,  5,  5,
        0,  0,  0, 20, 20,  0,  0,  0,
        5, -5,-10,  0,  0,-10, -5,  5,
        5, 10, 10,-20,-20, 10, 10,  5,
        0,  0,  0,  0,  0,  0,  0,  0,
    ],
];

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;

    for piece in Piece::ALL {
        for color in Color::ALL {
            let mut bb = board.bitboard(piece, color);

            for _ in 0..bb.0.count_ones() {
                let i = bb.pop_lsb();

                let pst_index = match color {
                    Color::White => 63 - i as usize,
                    Color::Black => i as usize,
                };

                let adjusted_score =
                    PIECE_SCORES[piece as usize] + PIECE_SQUARE_TABLES[piece as usize][pst_index];

                score += adjusted_score * color.direction() as i32;
            }
        }
    }

    score
}

#[cfg(test)]
pub mod eval_tests {
    use chress::move_gen::MoveGen;

    use super::*;

    #[test]
    fn eval_white_queen_down() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1",
            &move_gen,
        )
        .unwrap();

        println!("{}", evaluate(&board));
    }

    #[test]
    fn eval_white_big_center() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(
            "rnbq1k1r/1pppppbp/p5pn/8/2BPP3/2N2N2/PPP2PPP/R1BQK2R w KQ - 0 1",
            &move_gen,
        )
        .unwrap();

        println!("{}", evaluate(&board));
    }

    #[test]
    fn eval_white_big_center_but_queen_down() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(
            "rnbqkbnr/pppppppp/8/8/2BPPB2/2N2N2/PPP2PPP/3RR1K1 b kq - 0 1",
            &move_gen,
        )
        .unwrap();

        println!("{}", evaluate(&board));
    }
}
