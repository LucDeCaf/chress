#![feature(test)]

extern crate test;

#[cfg(test)]
mod board_tests {
    use chress::{
        bitboard::Bitboard,
        board::{Board, POSITION_2, POSITION_3, POSITION_4, POSITION_5, START_FEN},
        build::movemasks::KNIGHT_MOVES,
        color::Color,
        piece::Piece,
        r#move::Move,
        square::Square,
    };
    use rand::{thread_rng, Rng};
    use test::{black_box, Bencher};

    #[bench]
    fn friendly_pieces_offset(b: &mut Bencher) {
        let mut board = Board::default();

        b.iter(black_box(|| {
            board.friendly_pieces();
            board.active_color = board.active_color.inverse();
        }))
    }

    #[bench]
    fn enemy_pieces_offset(b: &mut Bencher) {
        let mut board = Board::default();

        b.iter(black_box(|| {
            board.enemy_pieces();
            board.active_color = board.active_color.inverse();
        }))
    }

    #[bench]
    fn legal_moves(b: &mut Bencher) {
        let mut board = Board::from_fen(POSITION_2).unwrap();

        b.iter(|| black_box(board.legal_moves()));
    }

    #[bench]
    fn append_moves_fn(b: &mut Bencher) {
        let board = Board::from_fen(START_FEN).unwrap();

        let mut moves = Vec::new();
        let mut color = Color::White;

        b.iter(|| {
            let pieces = board.bitboard(Piece::Knight, color);
            board.append_moves_getter(&mut moves, pieces, Board::knight_moves);

            color = color.inverse();
        });
    }

    #[bench]
    fn append_moves_fn_table(b: &mut Bencher) {
        let board = Board::from_fen(START_FEN).unwrap();

        let mut moves = Vec::new();
        let mut color = Color::White;

        b.iter(|| {
            let pieces = board.bitboard(Piece::Knight, color);
            board.append_moves_table(&mut moves, pieces, &KNIGHT_MOVES);

            color = color.inverse();
        });
    }

    #[bench]
    fn append_moves_inline(b: &mut Bencher) {
        let board = Board::from_fen(START_FEN).unwrap();

        let mut moves = Vec::new();
        let mut color = Color::White;

        b.iter(|| {
            let mut pieces = board.bitboard(Piece::Knight, color);

            for _ in 0..pieces.0.count_ones() {
                let i = pieces.pop_lsb();

                let from = Square::ALL[i as usize];
                let mut targets = board.knight_moves(from);

                for _ in 0..targets.0.count_ones() {
                    let j = targets.pop_lsb();
                    let to = Square::ALL[j as usize];

                    moves.push(Move::new(from, to));
                }
            }

            color = color.inverse();
        });
    }

    // 55 ± 1
    #[bench]
    fn moves_from_integrated(b: &mut Bencher) {
        let board = Board::from_fen(POSITION_2).unwrap();

        let mut color = Color::White;

        // Assign large arbitraty capacity to reduce chance of allocation taking up time
        let mut moves: Vec<Move> = Vec::with_capacity(2048);

        b.iter(|| {
            // Knight moves
            let mut knights = board.bitboard(Piece::Knight, Color::White);
            for _ in 0..knights.0.count_ones() {
                let i = knights.pop_lsb();

                let from = Square::ALL[i as usize];
                let mut targets = board.knight_moves(from);

                for _ in 0..targets.0.count_ones() {
                    let j = targets.pop_lsb();
                    let to = Square::ALL[j as usize];

                    moves.push(black_box(Move::new(from, to)));
                }
            }

            color = color.inverse()
        });
    }

    #[bench]
    fn make_unmake(b: &mut Bencher) {
        let mut board = Board::from_fen(POSITION_2).unwrap();
        let moves = board.legal_moves();

        b.iter(|| {
            for r#move in moves.iter() {
                board.make_move(*r#move).unwrap();
                board.unmake_move().unwrap();
            }
        })
    }

    // 30.7 ± 1.1
    #[bench]
    fn piece_at_branched(b: &mut Bencher) {
        let board = Board::from_fen(POSITION_2).unwrap();

        let mut rng = thread_rng();

        b.iter(|| {
            let square = Square::ALL[rng.gen_range(0..64)];

            for (i, bb) in board.piece_bitboards.into_iter().enumerate() {
                if !(bb & square.bitboard()).is_empty() {
                    return Some(Piece::ALL[i % 6]);
                }
            }

            None
        });
    }

    // 26.4 ± 0.7
    #[bench]
    fn piece_at_branchless(b: &mut Bencher) {
        let board = Board::from_fen(POSITION_2).unwrap();

        let mut rng = thread_rng();

        const PIECES: [Option<Piece>; 7] = [
            None,
            Some(Piece::Knight),
            Some(Piece::Bishop),
            Some(Piece::Rook),
            Some(Piece::Queen),
            Some(Piece::Pawn),
            Some(Piece::King),
        ];

        b.iter(|| {
            let square = Square::ALL[rng.gen_range(0..64)];

            let mask = square.bitboard();

            // Using conditionals, not branches
            let knights = !((board.piece_bitboards[0] | board.piece_bitboards[6]) & mask).is_empty()
                as usize
                * 1;
            let bishops = !((board.piece_bitboards[1] | board.piece_bitboards[7]) & mask).is_empty()
                as usize
                * 2;
            let rooks = !((board.piece_bitboards[2] | board.piece_bitboards[8]) & mask).is_empty()
                as usize
                * 3;
            let queens = !((board.piece_bitboards[3] | board.piece_bitboards[9]) & mask).is_empty()
                as usize
                * 4;
            let pawns = !((board.piece_bitboards[4] | board.piece_bitboards[10]) & mask).is_empty()
                as usize
                * 5;
            let kings = !((board.piece_bitboards[5] | board.piece_bitboards[11]) & mask).is_empty()
                as usize
                * 6;

            let piece_at_square_index = knights | bishops | rooks | queens | pawns | kings;

            PIECES[piece_at_square_index]
        });
    }

    #[test]
    fn perft_startpos() {
        let mut board = Board::from_fen(START_FEN).unwrap();

        assert_eq!(board.perft_parallel(6), 119060324);
    }

    #[test]
    fn perft_position_2() {
        let mut board = Board::from_fen(POSITION_2).unwrap();

        assert_eq!(board.perft_parallel(5), 193690690);
    }

    #[test]
    fn perft_position_3() {
        let mut board = Board::from_fen(POSITION_3).unwrap();

        assert_eq!(board.perft_parallel(7), 178633661);
    }

    #[test]
    fn perft_position_4() {
        let mut board = Board::from_fen(POSITION_4).unwrap();

        assert_eq!(board.perft_parallel(5), 15833292);
    }

    // 25,850,916.70 ns/iter (+/- 1,392,332.94)
    #[test]
    fn perft_position_5() {
        let mut board = Board::from_fen(POSITION_5).unwrap();

        assert_eq!(board.perft_parallel(5), 89941194);
    }

    #[test]
    fn fen_startpos() {
        let board = Board::from_fen(START_FEN).unwrap();

        assert_eq!(board.fen(), START_FEN);
    }

    #[test]
    fn fen_position_5() {
        let board = Board::from_fen(START_FEN).unwrap();

        assert_eq!(board.fen(), POSITION_5);
    }

    #[test]
    fn fen_en_passant() {
        const ONE_E4: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let board = Board::from_fen(ONE_E4).unwrap();

        assert_eq!(board.fen(), ONE_E4);
    }
}
