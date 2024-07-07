#![feature(test)]

extern crate test;

#[cfg(test)]
mod bitboard_tests {
    use chress::{
        bitboard::Bitboard,
        board::{Board, POSITION_2},
        color::Color,
        piece::Piece,
        r#move::Move,
        square::Square,
    };
    use test::Bencher;

    #[bench]
    fn append_moves_from_fn(b: &mut Bencher) {
        let mut board = Board::new();
        board.load_from_fen(POSITION_2).unwrap();

        let mut moves = Vec::new();

        let mut color = Color::White;

        b.iter(|| {
            let king_square =
                Square::ALL[board.bitboard(Piece::King, color).0.trailing_zeros() as usize];

            let mut targets = board.king_moves(king_square);

            targets.append_moves_from(&mut moves, king_square);

            color = color.inverse();
        });
    }

    #[bench]
    fn append_moves_from_inlined(b: &mut Bencher) {
        let mut board = Board::new();
        board.load_from_fen(POSITION_2).unwrap();

        let mut moves = Vec::new();

        let mut color = Color::White;

        b.iter(|| {
            let king_square =
                Square::ALL[board.bitboard(Piece::King, color).0.trailing_zeros() as usize];

            let mut targets = board.king_moves(king_square);

            for _ in 0..targets.0.count_ones() {
                moves.push(Move::new(
                    king_square,
                    Square::ALL[targets.pop_lsb() as usize],
                ));
            }

            color = color.inverse();
        });
    }

    #[test]
    fn bitboard_active() {
        let bb =
            Bitboard(0b10000000_00000000_00010000_00001110_00000000_00010000_00000000_00001011);
        let active = vec![
            Square::A1,
            Square::B1,
            Square::D1,
            Square::E3,
            Square::B5,
            Square::C5,
            Square::D5,
            Square::E6,
            Square::H8,
        ];

        assert_eq!(bb.active(), active);
    }
}
