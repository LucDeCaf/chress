#![feature(test)]

extern crate test;

#[cfg(test)]
mod board_tests {
    use chress::{
        bitboard::Bitboard,
        board::{Board, START_FEN},
        color::Color,
        piece::Piece,
        r#move::Move,
        square::Square,
    };
    use rand::{thread_rng, Rng};
    use test::Bencher;

    const POSITION_2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

    // Returns Result<(), E>: 1000 ± 40
    // Returns ()           : 1000 ± 60
    //
    // Will continue to return Results
    //
    // No lookup for castling rights:
    // Lookup for castling rights   :
    #[bench]
    fn make_unmake(b: &mut Bencher) {
        let mut board = Board::new();
        board.load_from_fen(POSITION_2).unwrap();
        let moves = board.legal_moves();

        b.iter(|| {
            for r#move in moves.iter() {
                board.make_move(*r#move).unwrap();
                board.unmake_move().unwrap();
            }
        })
    }

    // Position 3
    // - Branched: 4,600,000 ± 100,000
    // - Branchless: 4,600,000 ± 100,000
    //
    // Will use branchless simply because less branching is generally good
    #[bench]
    fn pseudo_en_passant(b: &mut Bencher) {
        let mut board = Board::new();
        board.load_from_fen(POSITION_3).unwrap();

        b.iter(|| board.perft(4))
    }

    // 30.7 ± 1.1
    #[bench]
    fn piece_at_branched(b: &mut Bencher) {
        let mut board = Board::new();

        // Load position with a lot of pieces
        board.load_from_fen(POSITION_2).unwrap();

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
        let mut board = Board::new();

        // Load position with a lot of pieces
        board.load_from_fen(POSITION_2).unwrap();

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
    fn white_pawn_movement() {
        let mut board = Board::new();

        // Setup
        board.add_piece(Piece::Pawn, Color::White, Square::E2);

        // Double jump
        assert_eq!(
            board.white_pawns_able_to_double_push(board.empty()),
            Square::E2.bitboard()
        );

        // Double jump blocked
        board.add_piece(Piece::Pawn, Color::Black, Square::E4);
        assert_eq!(
            board.white_pawns_able_to_push(board.empty()),
            Square::E2.bitboard()
        );
        assert_eq!(
            board.white_pawns_able_to_double_push(board.empty()),
            Bitboard::EMPTY
        );

        // Blocked
        board.add_piece(Piece::Pawn, Color::Black, Square::E3);
        assert_eq!(
            board.white_pawns_able_to_push(board.empty()),
            Bitboard::EMPTY
        );
    }

    #[test]
    fn black_pawn_movement() {
        let mut board = Board::new();

        // Setup
        board.add_piece(Piece::Pawn, Color::Black, Square::E7);

        // Double jump
        assert_eq!(
            board.black_pawns_able_to_double_push(board.empty()),
            Square::E7.bitboard()
        );

        // Double jump blocked
        board.add_piece(Piece::Pawn, Color::White, Square::E5);
        assert_eq!(
            board.black_pawns_able_to_push(board.empty()),
            Square::E7.bitboard()
        );
        assert_eq!(
            board.black_pawns_able_to_double_push(board.empty()),
            Bitboard::EMPTY
        );

        // Blocked
        board.add_piece(Piece::Pawn, Color::White, Square::E6);
        assert_eq!(
            board.black_pawns_able_to_push(board.empty()),
            Bitboard::EMPTY
        );
    }

    #[test]
    fn en_passant() {
        let mut board = Board::new();

        board.add_piece(Piece::Pawn, Color::White, Square::D5);
        board.add_piece(Piece::Pawn, Color::Black, Square::E7);

        board.active_color = Color::Black;

        board.make_move(Move::new(Square::E7, Square::E5)).unwrap();
        board.make_move(Move::new(Square::D5, Square::E6)).unwrap();

        assert_eq!(board.move_list[0].captured_piece, None);
        assert_eq!(board.move_list[1].captured_piece, Some(Piece::Pawn));

        assert_eq!(
            board.bitboard(Piece::Pawn, Color::White),
            Square::E6.bitboard()
        );
        assert_eq!(board.bitboard(Piece::Pawn, Color::Black), Bitboard::EMPTY);
    }

    #[test]
    fn make_unmake_simple() {
        let mut board_1 = Board::new();
        let mut board_2 = Board::new();

        assert_eq!(board_1, board_2);

        board_1.load_from_fen(START_FEN).unwrap();
        board_2.load_from_fen(START_FEN).unwrap();

        assert_eq!(board_1, board_2);

        let r#move = Move::new(Square::E2, Square::E4);

        board_1.make_move(r#move).unwrap();
        board_1
            .make_move(Move::new(Square::G8, Square::F6))
            .unwrap();
        board_1.unmake_move().unwrap();
        board_1.unmake_move().unwrap();

        assert_eq!(board_1, board_2);
    }

    #[test]
    fn make_unmake_en_passant() {
        const TEST_FEN: &str = "rnbqkbnr/ppp2ppp/8/3Pp3/8/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 3";

        let mut board_1 = Board::new();
        let mut board_2 = Board::new();

        assert_eq!(board_1, board_2);

        board_1.load_from_fen(TEST_FEN).unwrap();
        board_2.load_from_fen(TEST_FEN).unwrap();

        assert_eq!(board_1, board_2);

        let r#move = Move::new(Square::D5, Square::E6);

        board_1.make_move(r#move).unwrap();
        board_1.unmake_move().unwrap();

        assert_eq!(board_1, board_2);
    }

    #[test]
    fn make_unmake_scotch_opening() {
        let mut board = Board::new();
        board.load_from_fen(START_FEN).unwrap();

        const SCOTCH_GAME: &str =
            "e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 c6d4 d1d4 f8e7 e4e5 d7d5 e5d6 d8d6 d4d6 e7d6";

        for r#move in SCOTCH_GAME.split_ascii_whitespace() {
            board.make_move(Move::try_from(r#move).unwrap()).unwrap();
        }

        while let Some(_) = board.move_list.first() {
            println!("{}\n", board);
            board.unmake_move().unwrap();
        }

        println!("{}", board);

        let mut startpos = Board::new();
        startpos.load_from_fen(START_FEN).unwrap();

        assert_eq!(startpos, board)
    }

    #[test]
    fn perft_startpos() {
        let mut board = Board::new();
        board.load_from_fen(START_FEN).unwrap();

        assert_eq!(board.perft(0), 1);
        assert_eq!(board.perft(1), 20);
        assert_eq!(board.perft(2), 400);
        assert_eq!(board.perft(3), 8902);
        assert_eq!(board.perft(4), 197281);
        assert_eq!(board.perft(5), 4865609);
        assert_eq!(board.perft(6), 119060324);
        // assert_eq!(board.perft(7), 3195901860);
    }

    #[test]
    fn perft_position_2() {
        let mut board = Board::new();
        board.load_from_fen(POSITION_2).unwrap();

        assert_eq!(board.perft(1), 48);
        assert_eq!(board.perft(2), 2039);
        assert_eq!(board.perft(3), 97862);
        assert_eq!(board.perft(4), 4085603);
        assert_eq!(board.perft(5), 193690690);
        // assert_eq!(board.perft(6), 8031647685);
    }

    #[test]
    fn perft_position_3() {
        let mut board = Board::new();
        board.load_from_fen(POSITION_3).unwrap();

        assert_eq!(board.perft(1), 14);
        assert_eq!(board.perft(2), 191);
        assert_eq!(board.perft(3), 2812);
        assert_eq!(board.perft(4), 43238);
        assert_eq!(board.perft(5), 674624);
        assert_eq!(board.perft(6), 11030083);
        assert_eq!(board.perft(7), 178633661);
        // assert_eq!(board.perft(8), 3009794393);
    }

    #[test]
    fn perft_position_4() {
        let mut board = Board::new();
        board.load_from_fen(POSITION_4).unwrap();

        assert_eq!(board.perft(1), 6);
        assert_eq!(board.perft(2), 264);
        assert_eq!(board.perft(3), 9467);
        assert_eq!(board.perft(4), 422333);
        assert_eq!(board.perft(5), 15833292);
        // assert_eq!(board.perft(6), 706045033);
    }

    #[test]
    fn perft_position_5() {
        let mut board = Board::new();
        board.load_from_fen(POSITION_5).unwrap();

        assert_eq!(board.perft(1), 44);
        assert_eq!(board.perft(2), 1486);
        assert_eq!(board.perft(3), 62379);
        assert_eq!(board.perft(4), 2103487);
        assert_eq!(board.perft(5), 89941194);
    }

    #[test]
    fn fen_startpos() {
        let mut board = Board::new();
        board.load_from_fen(START_FEN).unwrap();

        assert_eq!(board.fen(), START_FEN);
    }

    #[test]
    fn fen_position_5() {
        let mut board = Board::new();
        board.load_from_fen(POSITION_5).unwrap();

        assert_eq!(board.fen(), POSITION_5);
    }

    #[test]
    fn fen_en_passant() {
        const ONE_E4: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let mut board = Board::new();
        board.load_from_fen(ONE_E4).unwrap();

        assert_eq!(board.fen(), ONE_E4);
    }
}
