use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Color {
    // Array containing the colors for branchless access
    pub const ALL: [Color; 2] = [Color::Black, Color::White];

    pub fn inverse(&self) -> Self {
        Self::ALL[(*self as usize) ^ 1]
    }

    pub fn direction(&self) -> i8 {
        2 * (*self as i8) - 1
    }

    pub fn en_passant_rank(&self) -> u8 {
        5 - (*self as u8 * 3)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const STRINGS: [&str; 2] = ["Black", "White"];
        write!(f, "{}", STRINGS[*self as usize])
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn inverse() {
        assert_eq!(Color::White.inverse(), Color::Black);
        assert_eq!(Color::Black.inverse(), Color::White);
    }

    #[test]
    fn direction() {
        assert_eq!(Color::White.direction(), 1);
        assert_eq!(Color::Black.direction(), -1);
    }

    #[test]
    fn en_passant_rank() {
        assert_eq!(Color::White.en_passant_rank(), 2);
        assert_eq!(Color::Black.en_passant_rank(), 5);
    }
}
