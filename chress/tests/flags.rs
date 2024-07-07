#[cfg(test)]
mod flag_tests {
    use chress::{color::Color, flags::Flags};

    #[test]
    fn kingside() {
        let flags = Flags(0b00000011);
        let color = Color::Black;

        assert!(flags.kingside(color));
        assert!(!flags.kingside(color.inverse()));
    }
}
