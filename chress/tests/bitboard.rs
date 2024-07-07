#![feature(test)]

extern crate test;

#[cfg(test)]
mod bitboard_tests {
    use std::hint::black_box;

    use chress::{
        bitboard::Bitboard,
        board::{Board, POSITION_2},
        color::Color,
        piece::Piece,
        r#move::Move,
        square::Square,
    };
    use rand::{thread_rng, Rng};
    use test::Bencher;

    #[bench]
    fn looped_bits_zero(b: &mut Bencher) {
        let bb = Bitboard(0);

        b.iter(|| {
            let mut count = 0;

            for sq in bb.into_iter() {
                count += black_box(sq as u8);
            }

            black_box(count)
        });
    }

    #[bench]
    fn looped_bits_max(b: &mut Bencher) {
        let bb = Bitboard(u64::MAX);

        b.iter(|| {
            let mut count = 0;

            for sq in bb.into_iter() {
                count += black_box(sq as u8);
            }

            black_box(count)
        });
    }

    #[bench]
    fn manual_bits_zero(b: &mut Bencher) {
        let mut bb = Bitboard(0);

        b.iter(|| {
            let mut count = 0;

            for _ in 0..bb.0.count_ones() {
                count += black_box(Square::ALL[bb.pop_lsb() as usize] as u8);
            }

            black_box(count)
        });
    }

    #[bench]
    fn manual_bits_max(b: &mut Bencher) {
        let mut bb = black_box(Bitboard(u64::MAX));

        b.iter(|| {
            black_box({
                let mut count = black_box(0);

                for _ in black_box(0..bb.0.count_ones()) {
                    black_box(count += Square::ALL[bb.pop_lsb() as usize] as u8);
                }

                black_box(count);
            });
        });
    }

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
