pub mod bitboard;
pub mod color;
pub mod flags;
pub mod r#move;
pub mod piece;
pub mod sliding_moves;
pub mod square;

use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    board::{
        bitboard::Bitboard,
        color::Color,
        flags::Flags,
        piece::Piece,
        r#move::{Move, MoveData},
        square::Square,
    },
    move_gen::MoveGen,
};

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const KING_STARTING_SQUARES: [Square; 2] = {
    let mut table = [Square::A1, Square::A1];

    table[Color::White as usize] = Square::E1;
    table[Color::Black as usize] = Square::E8;

    table
};
pub const CASTLING_BLOCKERS: [[Bitboard; 2]; 2] = {
    let mut table = [
        [Bitboard::EMPTY, Bitboard::EMPTY],
        [Bitboard::EMPTY, Bitboard::EMPTY],
    ];

    table[Color::White as usize][0] = Bitboard(Square::F1.bitboard().0 | Square::G1.bitboard().0);
    table[Color::White as usize][1] =
        Bitboard(Square::D1.bitboard().0 | Square::C1.bitboard().0 | Square::B1.bitboard().0);
    table[Color::Black as usize][0] = Bitboard(Square::F8.bitboard().0 | Square::G8.bitboard().0);
    table[Color::Black as usize][1] =
        Bitboard(Square::D8.bitboard().0 | Square::C8.bitboard().0 | Square::B8.bitboard().0);

    table
};
pub const CASTLING_CHECKABLES: [[Bitboard; 2]; 2] = {
    let mut table = [
        [Bitboard::EMPTY, Bitboard::EMPTY],
        [Bitboard::EMPTY, Bitboard::EMPTY],
    ];

    table[Color::White as usize][0] = Bitboard(Square::F1.bitboard().0 | Square::G1.bitboard().0);
    table[Color::White as usize][1] = Bitboard(Square::D1.bitboard().0 | Square::C1.bitboard().0);
    table[Color::Black as usize][0] = Bitboard(Square::F8.bitboard().0 | Square::G8.bitboard().0);
    table[Color::Black as usize][1] = Bitboard(Square::D8.bitboard().0 | Square::C8.bitboard().0);

    table
};
pub const CASTLING_DESTINATIONS: [[Square; 2]; 2] = {
    let mut table = [[Square::A1, Square::A1], [Square::A1, Square::A1]];

    table[Color::White as usize] = [Square::G1, Square::C1];
    table[Color::Black as usize] = [Square::G8, Square::C8];

    table
};
pub const CASTLING_RIGHTS_FLAGS: [Flags; 64] = {
    let mut table = [Flags::UNIVERSE; 64];

    table[Square::A1 as usize] = Flags(!Flags::WHITE_QUEENSIDE.0);
    table[Square::A8 as usize] = Flags(!Flags::BLACK_QUEENSIDE.0);
    table[Square::H1 as usize] = Flags(!Flags::WHITE_KINGSIDE.0);
    table[Square::H8 as usize] = Flags(!Flags::BLACK_KINGSIDE.0);

    table
};
pub const ROOK_CASTLING_MOVEMASKS: [Bitboard; 64] = {
    let mut table = [Bitboard::EMPTY; 64];
    table[Square::G1 as usize] = Bitboard(Square::H1.bitboard().0 | Square::F1.bitboard().0);
    table[Square::G8 as usize] = Bitboard(Square::H8.bitboard().0 | Square::F8.bitboard().0);
    table[Square::C1 as usize] = Bitboard(Square::A1.bitboard().0 | Square::D1.bitboard().0);
    table[Square::C8 as usize] = Bitboard(Square::A8.bitboard().0 | Square::D8.bitboard().0);
    table
};

#[derive(Debug)]
pub struct MakeMoveError;

impl Display for MakeMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to make move")
    }
}

impl Error for MakeMoveError {}

#[derive(Debug)]
pub struct UnmakeMoveError(String);

impl Display for UnmakeMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for UnmakeMoveError {}

#[derive(Debug, PartialEq)]
pub struct EngineOption {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct ParseOptionError;

impl Display for ParseOptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ParseOptionError {}

#[derive(Debug)]
pub enum ParseFenError {
    BadPosition,
    BadColor,
    BadCastlingRights,
    BadEnPassant,
    BadHalfmoves,
    BadFullmoves,
    WrongSectionCount,
    InvalidPosition,
}

impl Display for ParseFenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ParseFenError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub pieces: [Bitboard; 12],
    pub active_color: Color,

    pub flags: Flags,

    pub halfmoves: u32,
    pub fullmoves: u32,
}

impl Board {
    fn new() -> Self {
        Self {
            pieces: [Bitboard::EMPTY; 12],
            active_color: Color::White,

            flags: Flags(0),

            halfmoves: 0,
            fullmoves: 1,
        }
    }

    pub fn from_fen(fen: &str, move_gen: &MoveGen) -> Result<Self, ParseFenError> {
        let mut board = Board::new();
        board.load_from_fen(fen, move_gen)?;
        Ok(board)
    }

    pub fn load_from_fen(&mut self, fen: &str, move_gen: &MoveGen) -> Result<(), ParseFenError> {
        self.clear_bitboards();
        self.flags.0 = 0;

        let mut sections = fen.split(' ');

        let Some(position) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        let mut rank: i8 = 7;
        let mut file: i8 = 0;

        for char in position.chars() {
            match char {
                '0'..='8' => {
                    let digit = char.to_digit(9).unwrap() as i8;
                    file += digit;
                }
                'p' | 'n' | 'b' | 'r' | 'q' | 'k' | 'P' | 'N' | 'B' | 'R' | 'Q' | 'K' => {
                    let color = if char.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };

                    let piece = Piece::try_from(char).unwrap();

                    let square = Square::try_from(rank as usize * 8 + file as usize).unwrap();
                    self.add_piece(piece, color, square);

                    file += 1;
                }
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                _ => return Err(ParseFenError::BadPosition),
            }
        }

        // Check for kings
        if self.bitboard(Piece::King, Color::White).0.count_ones() != 1 {
            return Err(ParseFenError::InvalidPosition);
        }
        if self.bitboard(Piece::King, Color::Black).0.count_ones() != 1 {
            return Err(ParseFenError::InvalidPosition);
        }

        let Some(active_color) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        self.active_color = match active_color {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(ParseFenError::BadColor),
        };

        let enemy_king_index = self
            .bitboard(Piece::King, self.active_color.inverse())
            .0
            .trailing_zeros();
        let enemy_king_square = Square::ALL[enemy_king_index as usize];

        // If can capture opponent's king, position is invalid
        if move_gen.square_attacked_by(self, enemy_king_square, self.active_color) {
            return Err(ParseFenError::InvalidPosition);
        }

        let Some(castling_rights) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        if castling_rights != "-" {
            for right in castling_rights.chars() {
                match right {
                    'K' => self.flags |= Flags::WHITE_KINGSIDE,
                    'Q' => self.flags |= Flags::WHITE_QUEENSIDE,
                    'k' => self.flags |= Flags::BLACK_KINGSIDE,
                    'q' => self.flags |= Flags::BLACK_QUEENSIDE,
                    _ => return Err(ParseFenError::BadCastlingRights),
                }
            }
        }

        let Some(en_passant) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        // TODO: Implement checks to prevent invalid en passant squares
        if en_passant != "-" {
            // Allow en passant
            self.flags |= Flags::EP_IS_VALID;

            let Ok(square) = Square::try_from(en_passant) else {
                return Err(ParseFenError::BadEnPassant);
            };

            // Set en passant file
            self.flags.0 |= square.file() << 4;
        }

        let Some(halfmoves) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        self.halfmoves = if let Ok(halfmoves) = halfmoves.parse::<u32>() {
            halfmoves
        } else {
            return Err(ParseFenError::BadHalfmoves);
        };

        let Some(fullmoves) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        self.fullmoves = if let Ok(fullmoves) = fullmoves.parse::<u32>() {
            // Fullmoves can never be zero, as games start on move 1
            if fullmoves == 0 {
                return Err(ParseFenError::BadFullmoves);
            }
            fullmoves
        } else {
            return Err(ParseFenError::BadFullmoves);
        };

        Ok(())
    }

    pub fn fen(&self) -> String {
        let mut fen = String::new();

        let mut rank: i8 = 7;
        let mut file: i8 = 0;

        let mut tiles_since_last_piece: u8 = 0;
        loop {
            let square = Square::ALL[(rank * 8 + file) as usize];

            if let Some(piece) = self.piece_at(square) {
                if tiles_since_last_piece != 0 {
                    fen.push((tiles_since_last_piece + b'0') as char);
                }
                let mut ch = char::from(piece) as u8;
                if !(self.black_pieces() & square.bitboard()).is_empty() {
                    ch -= b'a' - b'A';
                }

                fen.push(ch as char);

                tiles_since_last_piece = 0;
            } else {
                tiles_since_last_piece += 1;
            }

            file += 1;
            if file == 8 {
                file = 0;
                rank -= 1;
                if rank == -1 {
                    break;
                } else {
                    if tiles_since_last_piece != 0 {
                        fen.push((tiles_since_last_piece + b'0') as char);
                    }
                    fen.push('/');
                    tiles_since_last_piece = 0;
                }
            }
        }

        fen.push(' ');

        fen.push(match self.active_color {
            Color::White => 'w',
            Color::Black => 'b',
        });

        fen.push(' ');

        let mut rights = String::new();

        if self.flags.white_kingside() {
            rights.push('K');
        }
        if self.flags.white_queenside() {
            rights.push('Q');
        }
        if self.flags.black_kingside() {
            rights.push('k');
        }
        if self.flags.black_queenside() {
            rights.push('q');
        }

        if rights.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&rights);
        }

        fen.push(' ');

        if let Some(file) = self.flags.en_passant_file() {
            let rank = self.active_color.inverse().en_passant_rank() + 1;
            let file = (file + b'a') as char;

            fen.push_str(&format!("{file}{rank}",));
        } else {
            fen.push('-');
        }

        fen.push_str(&format!(" {} {}", self.halfmoves, self.fullmoves));

        fen
    }

    pub fn flip_color(&mut self) {
        self.active_color = self.active_color.inverse();
    }

    fn clear_bitboards(&mut self) {
        for bb in &mut self.pieces {
            bb.0 = 0;
        }
    }

    pub fn bitboard(&self, piece: Piece, color: Color) -> Bitboard {
        self.pieces[Self::bitboard_index(piece, color)]
    }

    fn bitboard_mut(&mut self, piece: Piece, color: Color) -> &mut Bitboard {
        &mut self.pieces[Self::bitboard_index(piece, color)]
    }

    fn bitboard_index(piece: Piece, color: Color) -> usize {
        piece as usize + (color as usize * 6)
    }

    fn add_piece(&mut self, piece: Piece, color: Color, square: Square) {
        *self.bitboard_mut(piece, color) |= square.bitboard();
    }

    fn remove_piece(&mut self, piece: Piece, color: Color, square: Square) {
        *self.bitboard_mut(piece, color) &= !square.bitboard();
    }

    pub fn occupied(&self) -> Bitboard {
        self.white_pieces() | self.black_pieces()
    }

    pub fn white_pieces(&self) -> Bitboard {
        self.pieces[6]
            | self.pieces[7]
            | self.pieces[8]
            | self.pieces[9]
            | self.pieces[10]
            | self.pieces[11]
    }

    pub fn black_pieces(&self) -> Bitboard {
        self.pieces[0]
            | self.pieces[1]
            | self.pieces[2]
            | self.pieces[3]
            | self.pieces[4]
            | self.pieces[5]
    }

    pub fn white_bitboards(&self) -> &[Bitboard] {
        &self.pieces[6..12]
    }

    pub fn black_bitboards(&self) -> &[Bitboard] {
        &self.pieces[0..6]
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        const PIECES: [Option<Piece>; 7] = [
            None,
            Some(Piece::Knight),
            Some(Piece::Bishop),
            Some(Piece::Rook),
            Some(Piece::Queen),
            Some(Piece::King),
            Some(Piece::Pawn),
        ];

        let mask = square.bitboard();

        let knights = !((self.pieces[0] | self.pieces[6]) & mask).is_empty() as usize;
        let bishops = !((self.pieces[1] | self.pieces[7]) & mask).is_empty() as usize * 2;
        let rooks = !((self.pieces[2] | self.pieces[8]) & mask).is_empty() as usize * 3;
        let queens = !((self.pieces[3] | self.pieces[9]) & mask).is_empty() as usize * 4;
        let kings = !((self.pieces[4] | self.pieces[10]) & mask).is_empty() as usize * 5;
        let pawns = !((self.pieces[5] | self.pieces[11]) & mask).is_empty() as usize * 6;

        let piece_at_square_index = knights | bishops | rooks | queens | kings | pawns;

        PIECES[piece_at_square_index]
    }

    pub fn friendly_pieces(&self) -> Bitboard {
        let off = self.active_color as usize * 6;

        self.pieces[off]
            | self.pieces[off + 1]
            | self.pieces[off + 2]
            | self.pieces[off + 3]
            | self.pieces[off + 4]
            | self.pieces[off + 5]
    }

    pub fn enemy_pieces(&self) -> Bitboard {
        let off = self.active_color as usize * 6;

        self.pieces[6 - off]
            | self.pieces[7 - off]
            | self.pieces[8 - off]
            | self.pieces[9 - off]
            | self.pieces[10 - off]
            | self.pieces[11 - off]
    }

    pub fn empty(&self) -> Bitboard {
        !self.occupied()
    }

    /// Plays a move on the board.
    ///
    /// This function will fail if the From square does not contain a piece.
    pub fn make_move(&mut self, r#move: Move) -> Result<MoveData, MakeMoveError> {
        let color = self.active_color;
        let from = r#move.from();
        let to = r#move.to();
        let promotion = r#move.promotion();

        let Some(moved_piece) = self.piece_at(from) else {
            return Err(MakeMoveError);
        };

        // Create new move_data struct
        let mut move_data = MoveData {
            r#move,
            captured_piece: self.piece_at(to),
            halfmoves: self.halfmoves,
            flags: self.flags,
        };

        // Increment halfmoves
        // Will be overwritten if necessary
        self.halfmoves += 1;

        // Special pawn moves
        // TODO: Try to remove some branches here
        if moved_piece == Piece::Pawn {
            self.halfmoves = 0;

            let is_double_move = from.rank().abs_diff(to.rank()) == 2;

            // Unset en passant file bits if necessary
            self.flags &= !(Flags::EP_FILE * is_double_move);
            // Set ep flag and ep file data correctly
            self.flags |= (Flags::EP_IS_VALID | Flags(from.file() << 4)) * is_double_move;

            // En passant
            if !is_double_move {
                let is_en_passant = if let Some(file) = self.flags.en_passant_file() {
                    let rank = color.inverse().en_passant_rank();
                    let ep_mask = Bitboard(1 << (rank * 8 + file));
                    ep_mask == to.bitboard()
                } else {
                    false
                };

                self.flags &= !Flags::EP_IS_VALID;

                if is_en_passant {
                    let captured_piece_rank = from.rank();
                    let captured_piece_file = to.file();
                    let capture_square_index =
                        (captured_piece_rank * 8 + captured_piece_file) as usize;

                    let capture_square = Square::ALL[capture_square_index];

                    move_data.captured_piece = Some(Piece::Pawn);

                    self.remove_piece(Piece::Pawn, color.inverse(), capture_square);
                }
            }
        } else {
            self.flags &= !Flags::EP_IS_VALID;
        }

        // Special king moves
        let is_king_move = moved_piece == Piece::King;

        // Remove castling rights
        self.flags &= !(Flags(0b0000_0011 << (color as u8 * 2)) * is_king_move);

        let is_castling = is_king_move && from.file().abs_diff(to.file()) == 2;

        // Move rook if necessary
        let rook_move_mask = ROOK_CASTLING_MOVEMASKS[to as usize];
        *self.bitboard_mut(Piece::Rook, color) ^= rook_move_mask * is_castling;

        // Castling rights
        let is_rook = moved_piece == Piece::Rook;
        let reset_mask = Flags::UNIVERSE * !is_rook;

        self.flags &= CASTLING_RIGHTS_FLAGS[from as usize] | reset_mask;
        self.flags &= CASTLING_RIGHTS_FLAGS[to as usize];

        self.remove_piece(moved_piece, color, from);

        // If promotion, create a new piece of the correct type
        if let Some(promoted_piece) = promotion {
            self.add_piece(promoted_piece, color, to);
        }
        // Otherwise, just move the piece as normal
        else {
            self.add_piece(moved_piece, color, to);
        }

        // Remove any captured pieces
        if let Some(captured_piece) = move_data.captured_piece {
            self.remove_piece(captured_piece, color.inverse(), to);
        }

        // Swap colors
        self.active_color = self.active_color.inverse();

        // Update fullmove count
        self.fullmoves += color.inverse() as u32;

        Ok(move_data)
    }

    // ! 4 branches, but they may be irreplaceable / too expensive to remove
    /// Unmakes a move on the board by popping the most recent move data off the stack.
    ///
    /// This function will fail if there is no piece to unmove on the To square, or if there
    /// is no data on the stack to pop.
    pub fn unmake_move(&mut self, move_data: MoveData) -> Result<(), UnmakeMoveError> {
        let from = move_data.r#move.from();
        let to = move_data.r#move.to();
        let promotion = move_data.r#move.promotion();
        let color = self.active_color.inverse();

        let piece_at_to: Piece;

        // TODO: Try to remove branches, but honestly that seems unlikely
        let moved_piece = match promotion {
            Some(promoted_piece) => {
                piece_at_to = promoted_piece;

                Piece::Pawn
            }
            None => {
                let Some(moved_piece) = self.piece_at(to) else {
                    return Err(UnmakeMoveError(
                        "no piece at square ".to_owned() + &to.to_string(),
                    ));
                };

                piece_at_to = moved_piece;

                moved_piece
            }
        };

        self.add_piece(moved_piece, color, from);
        self.remove_piece(piece_at_to, color, to);

        // Move rook back if undoing castling
        let is_castling = moved_piece == Piece::King && from.file().abs_diff(to.file()) == 2;

        let rook_move_mask = ROOK_CASTLING_MOVEMASKS[to as usize];
        *self.bitboard_mut(Piece::Rook, color) ^= rook_move_mask * is_castling;

        // Replace any captured pieces
        if let Some(captured_piece) = move_data.captured_piece {
            let mut ep_mask = {
                let rank = color.inverse().en_passant_rank();
                let file = move_data.flags.en_passant_file_unchecked();
                Bitboard(1 << (rank * 8 + file))
            };

            let is_en_passant = move_data.flags.en_passant_valid() && ep_mask == to.bitboard();

            ep_mask = {
                let is_white = color == Color::White;

                let shl = (ep_mask << 8) * !is_white;
                let shr = (ep_mask >> 8) * is_white;

                shl | shr
            };

            let square_mask = (ep_mask * is_en_passant) | (to.bitboard() * !is_en_passant);

            *self.bitboard_mut(captured_piece, color.inverse()) |= square_mask;
        }

        // Set move data
        self.halfmoves = move_data.halfmoves;
        self.fullmoves -= self.active_color as u32;

        self.flags = move_data.flags;

        self.active_color = color;

        Ok(())
    }
}

impl Default for Board {
    /// Creates a new instance of Board with the starting position loaded.
    fn default() -> Self {
        Board {
            pieces: [
                // White knights
                Bitboard(66),
                // White bishops
                Bitboard(36),
                // White rooks
                Bitboard(129),
                // White queens
                Bitboard(8),
                // White kings
                Bitboard(16),
                // White pawns
                Bitboard(65280),
                // Black knights
                Bitboard(4755801206503243776),
                // Black bishops
                Bitboard(2594073385365405696),
                // Black rooks
                Bitboard(9295429630892703744),
                // Black queens
                Bitboard(576460752303423488),
                // Black kings
                Bitboard(1152921504606846976),
                // Black pawns
                Bitboard(71776119061217280),
            ],

            active_color: Color::White,
            flags: Flags(0b0000_1111),
            halfmoves: 0,
            fullmoves: 1,
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        const PIECE_CHARS: [char; 12] = [
            'n', 'b', 'r', 'q', 'k', 'p',
            'N', 'B', 'R', 'Q', 'K', 'P',
        ];

        let mut chars = [
            "8  . . . . . . . .\n".chars(),
            "7  . . . . . . . .\n".chars(),
            "6  . . . . . . . .\n".chars(),
            "5  . . . . . . . .\n".chars(),
            "4  . . . . . . . .\n".chars(),
            "3  . . . . . . . .\n".chars(),
            "2  . . . . . . . .\n".chars(),
            "1  . . . . . . . .\n\n".chars(),
            "   A B C D E F G H".chars(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<char>>();

        for (i, mut bb) in self.pieces.into_iter().enumerate() {
            let piece_char = PIECE_CHARS[i];

            for _ in 0..bb.0.count_ones() {
                let square = Square::ALL[bb.pop_lsb() as usize];

                let x_offset = 3 + ((square.file()) * 2) as usize;
                let y_offset = 19 * (7 - square.rank()) as usize;
                let index = x_offset + y_offset;
                chars[index] = piece_char;
            }
        }

        let mut display_string = String::with_capacity(chars.len());

        for ch in chars {
            display_string.push(ch);
        }

        write!(f, "{}", display_string)
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;

    // pub const KIWIPETE: &str =
    //     "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    // pub const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    // pub const POSITION_4: &str =
    //     "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    pub const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

    #[test]
    fn board_default() {
        assert_eq!(
            Board::from_fen(START_FEN, &MoveGen::new()).unwrap(),
            Board::default()
        );
    }

    #[test]
    fn fen_startpos() {
        let board = Board::default();

        assert_eq!(board.fen(), START_FEN);
    }

    #[test]
    fn fen_position_5() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(POSITION_5, &move_gen).unwrap();

        assert_eq!(board.fen(), POSITION_5);
    }

    #[test]
    fn fen_en_passant() {
        const ONE_E4: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let move_gen = MoveGen::new();
        let board = Board::from_fen(ONE_E4, &move_gen).unwrap();

        assert_eq!(board.fen(), ONE_E4);
    }
}
