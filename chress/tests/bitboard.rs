#[cfg(test)]
mod bitboard_tests {
    use chress::{bitboard::Bitboard, square::Square};

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
