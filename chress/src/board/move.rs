use std::{cmp::Ordering, fmt::Display};

use serde::{Deserialize, Serialize};

use super::{flags::Flags, piece::Piece, square::Square};

#[derive(Debug)]
pub enum ParseMoveError {
    BadFrom,
    BadTo,
    BadPromotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move(u16);

impl Move {
    pub const PROMOTION_MASK: u16 = 0b0000_0000_0000_1111;

    pub const KS_WHITE: Move = Move::new(Square::E1, Square::G1);
    pub const QS_WHITE: Move = Move::new(Square::E1, Square::C1);
    pub const KS_BLACK: Move = Move::new(Square::E8, Square::G8);
    pub const QS_BLACK: Move = Move::new(Square::E8, Square::C8);

    pub const NULLMOVE: Move = Move(0);

    pub const fn new(from: Square, to: Square) -> Self {
        let from = from as u16;
        let to = to as u16;

        let mask = (from << 10) | (to << 4);

        Self(mask)
    }

    pub const fn new_with_promotion(from: Square, to: Square, promotion: Piece) -> Self {
        let from = from as u16;
        let to = to as u16;
        let promotion = promotion.promotion_mask();

        let mask = (from << 10) | (to << 4) | promotion;

        Self(mask)
    }

    pub const fn new_with_possible_promotion(
        from: Square,
        to: Square,
        promotion: Option<Piece>,
    ) -> Self {
        match promotion {
            Some(promotion) => Self::new_with_promotion(from, to, promotion),
            None => Self::new(from, to),
        }
    }

    pub const fn from(&self) -> Square {
        let square_index = self.0 >> 10;
        Square::ALL[square_index as usize]
    }

    pub const fn to(&self) -> Square {
        let square_index = (self.0 >> 4) & 0b111111;
        Square::ALL[square_index as usize]
    }

    pub const fn promotion(&self) -> Option<Piece> {
        const LOOKUP: [Option<Piece>; 9] = [
            None,
            Some(Piece::Knight),
            Some(Piece::Bishop),
            None,
            Some(Piece::Rook),
            None,
            None,
            None,
            Some(Piece::Queen),
        ];

        let promotion_index = self.0 & Self::PROMOTION_MASK;

        LOOKUP[promotion_index as usize]
    }
}

// Only used for display purposes, does not need to be branchless
impl PartialOrd for Move {
    fn ge(&self, other: &Self) -> bool {
        let f1 = self.from();
        let f2 = other.from();
        let t1 = self.to();
        let t2 = other.to();
        let p1 = self.promotion();
        let p2 = other.promotion();

        if f1 == f2 {
            if t1 == t2 {
                if let Some(p1) = p1 {
                    if let Some(p2) = p2 {
                        p1 >= p2
                    } else {
                        // self has one, other has none
                        // therefore self is greater
                        true
                    }
                } else if let Some(_p2) = p2 {
                    // self has none, other has one
                    // therefore self is not greater
                    false
                } else {
                    // They are equal
                    true
                }
            } else {
                t1 >= t2
            }
        } else {
            f1 >= f2
        }
    }

    fn gt(&self, other: &Self) -> bool {
        let f1 = self.from();
        let f2 = other.from();
        let t1 = self.to();
        let t2 = other.to();
        let p1 = self.promotion();
        let p2 = other.promotion();

        if f1 == f2 {
            if t1 == t2 {
                if let Some(p1) = p1 {
                    if let Some(p2) = p2 {
                        p1 > p2
                    } else {
                        // self has one, other has none
                        // therefore self is greater
                        true
                    }
                } else if let Some(_p2) = p2 {
                    // self has none, other has one
                    // therefore self is not greater
                    false
                } else {
                    // They are equal
                    false
                }
            } else {
                t1 > t2
            }
        } else {
            f1 > f2
        }
    }

    fn le(&self, other: &Self) -> bool {
        let f1 = self.from();
        let f2 = other.from();
        let t1 = self.to();
        let t2 = other.to();
        let p1 = self.promotion();
        let p2 = other.promotion();

        if f1 == f2 {
            if t1 == t2 {
                if let Some(p1) = p1 {
                    if let Some(p2) = p2 {
                        p1 <= p2
                    } else {
                        // self has one, other has none
                        // therefore self is not lesser
                        false
                    }
                } else if let Some(_p2) = p2 {
                    // self has none, other has one
                    // therefore self is lesser
                    true
                } else {
                    // They are equal
                    true
                }
            } else {
                t1 <= t2
            }
        } else {
            f1 <= f2
        }
    }

    fn lt(&self, other: &Self) -> bool {
        let f1 = self.from();
        let f2 = other.from();
        let t1 = self.to();
        let t2 = other.to();
        let p1 = self.promotion();
        let p2 = other.promotion();

        if f1 == f2 {
            if t1 == t2 {
                if let Some(p1) = p1 {
                    if let Some(p2) = p2 {
                        p1 < p2
                    } else {
                        // self has one, other has none
                        // therefore self is not lesser
                        false
                    }
                } else if let Some(_p2) = p2 {
                    // self has none, other has one
                    // therefore self is lesser
                    true
                } else {
                    // They are equal
                    false
                }
            } else {
                t1 < t2
            }
        } else {
            f1 < f2
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Move {
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        let promotion = match self.promotion() {
            Some(piece) => match min.promotion() {
                Some(min_piece) => match max.promotion() {
                    Some(max_piece) => {
                        if min_piece > piece {
                            Some(min_piece)
                        } else if max_piece < piece {
                            Some(max_piece)
                        } else {
                            Some(piece)
                        }
                    }
                    None => None,
                },
                None => None,
            },
            None => None,
        };

        Self::new_with_possible_promotion(
            Square::clamp(self.from(), min.from(), max.from()),
            Square::clamp(self.to(), min.to(), max.to()),
            promotion,
        )
    }

    fn cmp(&self, other: &Self) -> Ordering {
        #[allow(clippy::comparison_chain)]
        if self > other {
            Ordering::Greater
        } else if self < other {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self > other {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        if self < other {
            self
        } else {
            other
        }
    }
}

impl TryFrom<&str> for Move {
    type Error = ParseMoveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let Ok(from) = Square::try_from(&value[0..2]) else {
            return Err(ParseMoveError::BadFrom);
        };
        let Ok(to) = Square::try_from(&value[2..4]) else {
            return Err(ParseMoveError::BadTo);
        };
        let promotion = match value.chars().nth(4) {
            Some(promotion_char) => match Piece::try_from(promotion_char) {
                Ok(piece) => Some(piece),
                Err(_) => return Err(ParseMoveError::BadPromotion),
            },
            None => None,
        };

        Ok(Self::new_with_possible_promotion(from, to, promotion))
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion_char = match self.promotion() {
            Some(piece) => char::from(piece),
            None => ' ',
        };

        write!(f, "{}{}{}", self.from(), self.to(), promotion_char)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveData {
    pub r#move: Move,

    pub captured_piece: Option<Piece>,

    pub flags: Flags,

    pub halfmoves: u32,
}
