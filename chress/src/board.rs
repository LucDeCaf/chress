use std::{error::Error, fmt::Display};

use crate::{
    bitboard::Bitboard,
    color::Color,
    flags::{self, Flags},
    piece::Piece,
    r#move::{Move, MoveData},
    square::Square,
};

use crate::build::{
    magics::{BISHOP_MAGICS, ROOK_MAGICS},
    movemasks::{KING_MOVES, KNIGHT_MOVES, PAWN_CAPTURES},
};

use crate::move_gen::{create_bishop_table, create_rook_table, magic_index};

pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const KING_STARTING_SQUARES: [Square; 2] = [Square::E8, Square::E1];
const CASTLING_BLOCKERS: [[Bitboard; 2]; 2] = [
    // Black
    [
        Bitboard(Square::F8.bitboard().0 | Square::G8.bitboard().0), // Kingside
        Bitboard(Square::D8.bitboard().0 | Square::C8.bitboard().0 | Square::B8.bitboard().0), // Queenside
    ],
    // White
    [
        Bitboard(Square::F1.bitboard().0 | Square::G1.bitboard().0), // Kingside
        Bitboard(Square::D1.bitboard().0 | Square::C1.bitboard().0 | Square::B1.bitboard().0), // Queenside
    ],
];
const CASTLING_CHECKABLES: [[Bitboard; 2]; 2] = [
    // Black
    [
        Bitboard(Square::F8.bitboard().0 | Square::G8.bitboard().0), // Kingside
        Bitboard(Square::D8.bitboard().0 | Square::C8.bitboard().0), // Queenside
    ],
    // White
    [
        Bitboard(Square::F1.bitboard().0 | Square::G1.bitboard().0), // Kingside
        Bitboard(Square::D1.bitboard().0 | Square::C1.bitboard().0), // Queenside
    ],
];
const CASTLING_DESTINATIONS: [[Square; 2]; 2] = [
    // Black
    [
        Square::G8, // Kingside
        Square::C8, // Queenside
    ],
    // White
    [
        Square::G1, // Kingside
        Square::C1, // Queenside
    ],
];
const CASTLING_RIGHTS_FLAGS: [Flags; 64] = {
    let mut table = [Flags::UNIVERSE; 64];
    table[Square::A1 as usize] = Flags(!flags::masks::WHITE_QUEENSIDE.0);
    table[Square::A8 as usize] = Flags(!flags::masks::BLACK_QUEENSIDE.0);
    table[Square::H1 as usize] = Flags(!flags::masks::WHITE_KINGSIDE.0);
    table[Square::H8 as usize] = Flags(!flags::masks::BLACK_KINGSIDE.0);
    table
};
const ROOK_CASTLING_MOVEMASKS: [Bitboard; 64] = {
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
}

impl Display for ParseFenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for ParseFenError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub piece_bitboards: [Bitboard; 12],

    pub active_color: Color,

    pub flags: Flags,

    pub move_list: Vec<MoveData>,

    pub halfmoves: u32,
    pub fullmoves: u32,

    rook_move_table: Vec<Bitboard>,
    bishop_move_table: Vec<Bitboard>,
}

impl Board {
    pub const FILE_A: Bitboard = Bitboard(0x0101010101010101);
    pub const FILE_H: Bitboard = Bitboard(0x8080808080808080);
    pub const RANK_1: Bitboard = Bitboard(0x00000000000000FF);
    pub const RANK_8: Bitboard = Bitboard(0xFF00000000000000);

    pub fn new() -> Self {
        Self {
            piece_bitboards: [Bitboard::EMPTY; 12],

            active_color: Color::White,

            flags: Flags(0),

            move_list: Vec::new(),

            halfmoves: 0,
            fullmoves: 1,

            rook_move_table: create_rook_table(),
            bishop_move_table: create_bishop_table(),
        }
    }

    pub fn load_from_fen(&mut self, fen: &str) -> Result<(), ParseFenError> {
        self.clear_bitboards();
        self.move_list.clear();
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

        let Some(active_color) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        self.active_color = match active_color {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(ParseFenError::BadColor),
        };

        let Some(castling_rights) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        if castling_rights != "-" {
            for right in castling_rights.chars() {
                match right {
                    'K' => self.flags |= flags::masks::WHITE_KINGSIDE,
                    'Q' => self.flags |= flags::masks::WHITE_QUEENSIDE,
                    'k' => self.flags |= flags::masks::BLACK_KINGSIDE,
                    'q' => self.flags |= flags::masks::BLACK_QUEENSIDE,
                    _ => return Err(ParseFenError::BadCastlingRights),
                }
            }
        }

        let Some(en_passant) = sections.next() else {
            return Err(ParseFenError::WrongSectionCount);
        };

        if en_passant != "-" {
            // Allow en passant
            self.flags |= flags::masks::EP_IS_VALID;

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
                if !(self.white_pieces() & square.bitboard()).is_empty() {
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
            let rank = 3 + (self.active_color as u8 * 3);
            let file = (file + b'a') as char;

            fen.push_str(&format!("{file}{rank}",));
        } else {
            fen.push('-');
        }

        fen.push_str(&format!(" {} {}", self.halfmoves, self.fullmoves));

        fen
    }

    pub fn perft(&mut self, depth: usize) -> u64 {
        let mut results = 0;

        let legal_moves = self.legal_moves();

        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            return legal_moves.len() as u64;
        }

        for r#move in self.legal_moves() {
            self.make_move(r#move).unwrap();
            results += self.perft(depth - 1);
            self.unmake_move().unwrap();
        }

        results
    }

    pub fn divide(&mut self, depth: usize) -> (u64, Vec<(Move, u64)>) {
        let mut total = 0;
        let moves = self.legal_moves();
        let mut results = Vec::with_capacity(moves.len());

        for r#move in moves {
            self.make_move(r#move).unwrap();

            let count = self.perft(depth - 1);
            total += count;
            results.push((r#move, count));

            self.unmake_move().unwrap();
        }

        results.sort_by(|(a, _), (b, _)| a.cmp(b));

        (total, results)
    }

    pub fn clear_bitboards(&mut self) {
        for bb in &mut self.piece_bitboards {
            bb.0 = 0;
        }
    }

    pub fn bitboard(&self, piece: Piece, color: Color) -> Bitboard {
        self.piece_bitboards[Self::bitboard_index(piece, color)]
    }

    pub fn bitboard_mut(&mut self, piece: Piece, color: Color) -> &mut Bitboard {
        &mut self.piece_bitboards[Self::bitboard_index(piece, color)]
    }

    pub fn bitboard_index(piece: Piece, color: Color) -> usize {
        piece as usize + (color as usize * 6)
    }

    pub fn add_piece(&mut self, piece: Piece, color: Color, square: Square) {
        *self.bitboard_mut(piece, color) |= square.bitboard();
    }

    pub fn remove_piece(&mut self, piece: Piece, color: Color, square: Square) {
        *self.bitboard_mut(piece, color) &= !square.bitboard();
    }

    pub fn occupied(&self) -> Bitboard {
        let mut mask = Bitboard::EMPTY;
        for bb in self.piece_bitboards {
            mask |= bb;
        }
        mask
    }

    pub fn white_pieces(&self) -> Bitboard {
        let mut mask = Bitboard::EMPTY;
        for bb in self.white_bitboards() {
            mask |= bb;
        }
        mask
    }

    pub fn black_pieces(&self) -> Bitboard {
        let mut mask = Bitboard::EMPTY;
        for bb in self.black_bitboards() {
            mask |= bb;
        }
        mask
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        const PIECES: [Option<Piece>; 7] = [
            None,
            Some(Piece::Knight),
            Some(Piece::Bishop),
            Some(Piece::Rook),
            Some(Piece::Queen),
            Some(Piece::Pawn),
            Some(Piece::King),
        ];

        let mask = square.bitboard();

        // Using conditionals, not branches
        let knights =
            !((self.piece_bitboards[0] | self.piece_bitboards[6]) & mask).is_empty() as usize * 1;
        let bishops =
            !((self.piece_bitboards[1] | self.piece_bitboards[7]) & mask).is_empty() as usize * 2;
        let rooks =
            !((self.piece_bitboards[2] | self.piece_bitboards[8]) & mask).is_empty() as usize * 3;
        let queens =
            !((self.piece_bitboards[3] | self.piece_bitboards[9]) & mask).is_empty() as usize * 4;
        let pawns =
            !((self.piece_bitboards[4] | self.piece_bitboards[10]) & mask).is_empty() as usize * 5;
        let kings =
            !((self.piece_bitboards[5] | self.piece_bitboards[11]) & mask).is_empty() as usize * 6;

        let piece_at_square_index = knights | bishops | rooks | queens | pawns | kings;

        PIECES[piece_at_square_index]
    }

    pub fn white_pawns_able_to_push(&self, empty: Bitboard) -> Bitboard {
        (empty >> 8) & self.bitboard(Piece::Pawn, Color::White)
    }

    pub fn black_pawns_able_to_push(&self, empty: Bitboard) -> Bitboard {
        (empty << 8) & self.bitboard(Piece::Pawn, Color::Black)
    }

    pub fn white_pawns_able_to_double_push(&self, empty: Bitboard) -> Bitboard {
        const RANK_4: Bitboard = Bitboard(0x00000000FF000000);
        let empty_in_rank_3 = ((empty & RANK_4) >> 8) & empty;
        self.white_pawns_able_to_push(empty_in_rank_3)
    }

    pub fn black_pawns_able_to_double_push(&self, empty: Bitboard) -> Bitboard {
        const RANK_5: Bitboard = Bitboard(0x000000FF00000000);
        let empty_in_rank_6 = ((empty & RANK_5) << 8) & empty;
        self.black_pawns_able_to_push(empty_in_rank_6)
    }

    pub fn white_bitboards(&self) -> Vec<Bitboard> {
        self.piece_bitboards[6..12].to_vec()
    }

    pub fn black_bitboards(&self) -> Vec<Bitboard> {
        self.piece_bitboards[0..6].to_vec()
    }

    pub fn friendly_bitboards(&self) -> Vec<Bitboard> {
        let offset = self.active_color as usize * 6;
        self.piece_bitboards[offset..offset + 6].to_vec()
    }

    pub fn enemy_bitboards(&self) -> Vec<Bitboard> {
        let offset = self.active_color.inverse() as usize * 6;
        self.piece_bitboards[offset..offset + 6].to_vec()
    }

    pub fn friendly_pieces(&self) -> Bitboard {
        let mut mask = Bitboard::EMPTY;
        for bb in self.friendly_bitboards() {
            mask |= bb;
        }
        mask
    }

    pub fn enemy_pieces(&self) -> Bitboard {
        let mut mask = Bitboard::EMPTY;
        for bb in self.enemy_bitboards() {
            mask |= bb;
        }
        mask
    }

    pub fn empty(&self) -> Bitboard {
        !(self.friendly_pieces() | self.enemy_pieces())
    }

    /// Squares seen by a rook on square
    pub fn rook_attacks(&self, square: Square, blockers: Bitboard) -> Bitboard {
        self.rook_move_table[magic_index(&ROOK_MAGICS[square as usize], blockers)]
    }

    /// Squares seen by a bishop on square
    pub fn bishop_attacks(&self, square: Square, blockers: Bitboard) -> Bitboard {
        self.bishop_move_table[magic_index(&BISHOP_MAGICS[square as usize], blockers)]
    }

    /// Squares seen by a queen on square
    pub fn queen_attacks(&self, square: Square, blockers: Bitboard) -> Bitboard {
        self.rook_attacks(square, blockers) | self.bishop_attacks(square, blockers)
    }

    /// Squares seen by a pawn on square
    pub fn pawn_attacks(&self, square: Square, color: Color) -> Bitboard {
        PAWN_CAPTURES[color as usize][square as usize]
    }

    /// Squares seen by a knight on square
    pub fn knight_attacks(square: Square) -> Bitboard {
        KNIGHT_MOVES[square as usize]
    }

    // Squares seen by a king on square
    pub fn king_attacks(square: Square) -> Bitboard {
        KING_MOVES[square as usize]
    }

    pub fn rook_moves(&self, square: Square) -> Bitboard {
        let friendly_pieces = self.friendly_pieces();
        let enemy_pieces = self.enemy_pieces();

        self.rook_attacks(square, friendly_pieces | enemy_pieces) & !friendly_pieces
    }

    pub fn bishop_moves(&self, square: Square) -> Bitboard {
        let friendly = self.friendly_pieces();
        let enemy = self.enemy_pieces();

        self.bishop_attacks(square, friendly | enemy) & !friendly
    }

    pub fn queen_moves(&self, square: Square) -> Bitboard {
        let friendly_pieces = self.friendly_pieces();
        let enemy_pieces = self.enemy_pieces();
        let blockers = friendly_pieces | enemy_pieces;

        let attacks = self.rook_attacks(square, blockers) | self.bishop_attacks(square, blockers);

        attacks & !friendly_pieces
    }

    pub fn knight_moves(&self, square: Square) -> Bitboard {
        Self::knight_attacks(square) & !self.friendly_pieces()
    }

    pub fn king_moves(&self, square: Square) -> Bitboard {
        Self::king_attacks(square) & !self.friendly_pieces()
    }

    /// Get all pseudolegal moves
    pub fn pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let color = self.active_color;
        let attacker_color = color.inverse();

        let friendly_pieces = self.friendly_pieces();
        let enemy_pieces = self.enemy_pieces();

        let all_pieces = friendly_pieces | enemy_pieces;
        let empty_squares = !all_pieces;

        // Pawn move data
        // ! One branch for all pawn move data
        let (single_push_froms, double_push_froms) = match color {
            Color::White => (
                self.white_pawns_able_to_push(empty_squares),
                self.white_pawns_able_to_double_push(empty_squares),
            ),
            Color::Black => (
                self.black_pawns_able_to_push(empty_squares),
                self.black_pawns_able_to_double_push(empty_squares),
            ),
        };

        let pawns = self.bitboard(Piece::Pawn, color);

        // Single moves
        for from in single_push_froms.active() {
            let to_index = from as i8 + (8 * color.direction());
            let to = Square::try_from(to_index as usize).unwrap();

            // Promotion
            if to.rank() % 7 == 0 {
                moves.push(Move::new_with_promotion(from, to, Piece::Knight));
                moves.push(Move::new_with_promotion(from, to, Piece::Bishop));
                moves.push(Move::new_with_promotion(from, to, Piece::Rook));
                moves.push(Move::new_with_promotion(from, to, Piece::Queen));
            } else {
                moves.push(Move::new(from, to));
            }
        }

        // Double moves
        for from in double_push_froms.active() {
            let to_index = from as isize + (16 * color.direction()) as isize;
            let to = Square::ALL[to_index as usize];

            moves.push(Move::new(from, to));
        }

        // Captures
        for from in pawns.active() {
            let captures = self.pawn_attacks(from, color) & enemy_pieces;

            // Promotion
            for to in captures.active() {
                if to.rank() % 7 == 0 {
                    moves.push(Move::new_with_promotion(from, to, Piece::Knight));
                    moves.push(Move::new_with_promotion(from, to, Piece::Bishop));
                    moves.push(Move::new_with_promotion(from, to, Piece::Rook));
                    moves.push(Move::new_with_promotion(from, to, Piece::Queen));
                } else {
                    moves.push(Move::new(from, to));
                }
            }
        }

        let rank = color.inverse().en_passant_rank();
        let file = self.flags.en_passant_file_unchecked();

        let ep_square = Square::ALL[(rank * 8 + file) as usize];

        let can_en_passant = self.flags.en_passant_valid();

        // Apply the inverse of this mask if can't en passant to remove the
        // possibility that any pawns will be found
        let block_mask = Bitboard(Bitboard::UNIVERSE.0 * !can_en_passant as u64);

        let pawns_that_can_take =
            self.pawn_attacks(ep_square, color.inverse()) & self.bitboard(Piece::Pawn, color);

        for from in (pawns_that_can_take & !block_mask).active() {
            moves.push(Move::new(from, ep_square));
        }

        // Knight moves
        let knights = self.bitboard(Piece::Knight, color);
        for from in knights.active() {
            moves.append(&mut self.knight_moves(from).moves_from(from));
        }

        // King moves
        let kings = self.bitboard(Piece::King, color);
        for from in kings.active() {
            moves.append(&mut self.king_moves(from).moves_from(from));
        }

        // Castling
        // Check if king is on start square and not in check
        let king_square = KING_STARTING_SQUARES[color as usize];
        let on_start_square = !(kings & king_square.bitboard()).is_empty();
        let in_check = self.square_attacked_by(king_square, attacker_color);

        if on_start_square && !in_check {
            let blocker_list = CASTLING_BLOCKERS[color as usize];
            let targets = CASTLING_DESTINATIONS[color as usize];
            let allowed = [self.flags.kingside(color), self.flags.queenside(color)];

            'outer: for i in 0..2 {
                // Disallow castling if it is disallowed (omg so smart)
                if !allowed[i] {
                    continue;
                }

                let blockers = blocker_list[i];

                // Check for pieces in the way
                if !(blockers & self.occupied()).is_empty() {
                    continue;
                }

                // Check if castling through/out of check
                // Don't need to check if castling into check as that is checked
                // in legal_moves already (would be redundant)
                for square in CASTLING_CHECKABLES[color as usize][i].active() {
                    if self.square_attacked_by(square, attacker_color) {
                        continue 'outer;
                    }
                }

                // Add castling as pseudolegal move
                moves.push(Move::new(king_square, targets[i]));
            }
        }

        // Rook moves
        let rooks = self.bitboard(Piece::Rook, color);
        for from in rooks.active() {
            moves.append(&mut self.rook_moves(from).moves_from(from));
        }

        // Bishop moves
        let bishops = self.bitboard(Piece::Bishop, color);
        for from in bishops.active() {
            moves.append(&mut self.bishop_moves(from).moves_from(from));
        }

        // Queen moves
        let queens = self.bitboard(Piece::Queen, color);
        for from in queens.active() {
            moves.append(&mut self.queen_moves(from).moves_from(from));
        }

        moves
    }

    /// Gets all legal moves.
    ///
    /// Takes a mutable reference to self because to check legality each
    /// move is made and unmade on the board before checking if the king
    /// is under attack.
    pub fn legal_moves(&mut self) -> Vec<Move> {
        let pseudolegal_moves = self.pseudolegal_moves();

        pseudolegal_moves
            .into_iter()
            .filter(|r#move| self.is_legal_move(*r#move))
            .collect()
    }

    /// Takes a mutable reference to self because to check legality the
    /// move is made and unmade on the board before checking if the king
    /// is under attack.
    pub fn is_legal_move(&mut self, r#move: Move) -> bool {
        let current_color = self.active_color;
        let attacker_color = current_color.inverse();

        self.make_move(r#move).unwrap();

        let king_square =
            Square::try_from(self.bitboard(Piece::King, current_color).0.trailing_zeros() as usize)
                .unwrap();

        let is_legal = !self.square_attacked_by(king_square, attacker_color);

        self.unmake_move().unwrap();

        is_legal
    }

    /// Checks if a square is seen by pieces of a certain color for the
    /// purpose of legal move generation
    pub fn square_attacked_by(&self, square: Square, attacker_color: Color) -> bool {
        let pawn_attackers = self.pawn_attacks(square, attacker_color.inverse())
            & self.bitboard(Piece::Pawn, attacker_color);

        if !pawn_attackers.is_empty() {
            return true;
        }

        let king_attacks = KING_MOVES[square as usize] & self.bitboard(Piece::King, attacker_color);

        if !king_attacks.is_empty() {
            return true;
        }

        let knight_attacks =
            KNIGHT_MOVES[square as usize] & self.bitboard(Piece::Knight, attacker_color);

        if !knight_attacks.is_empty() {
            return true;
        }

        let rook_attacks = self.rook_attacks(square, self.occupied());
        let rooks_queens = self.bitboard(Piece::Rook, attacker_color)
            | self.bitboard(Piece::Queen, attacker_color);

        if !(rook_attacks & rooks_queens).is_empty() {
            return true;
        }

        let bishop_attacks = self.bishop_attacks(square, self.occupied());
        let bishops_queens = self.bitboard(Piece::Bishop, attacker_color)
            | self.bitboard(Piece::Queen, attacker_color);

        if !(bishop_attacks & bishops_queens).is_empty() {
            return true;
        }

        false
    }

    /// Returns the captured piece, if any
    pub fn make_move(&mut self, r#move: Move) -> Result<(), MakeMoveError> {
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

            // Double moves (en passant mask)
            if from.rank().abs_diff(to.rank()) == 2 {
                // Allow en passant
                self.flags |= flags::masks::EP_IS_VALID;

                // Unset file bits
                self.flags &= !flags::masks::EP_FILE;

                // Set file bits to correct file
                self.flags.0 |= from.file() << 4;
            }
            // En passant
            else {
                let is_en_passant = if let Some(file) = self.flags.en_passant_file() {
                    let rank = color.inverse().en_passant_rank();
                    let ep_mask = Bitboard(1 << (rank * 8 + file));
                    ep_mask == to.bitboard()
                } else {
                    false
                };

                self.flags &= !flags::masks::EP_IS_VALID;

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
            self.flags &= !flags::masks::EP_IS_VALID;
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

        // Update move list
        self.move_list.push(move_data);

        Ok(())
    }

    pub fn unmake_move(&mut self) -> Result<(), UnmakeMoveError> {
        let Some(move_data) = self.move_list.pop() else {
            return Err(UnmakeMoveError("no move to unmake".to_owned()));
        };

        let from = move_data.r#move.from();
        let to = move_data.r#move.to();
        let promotion = move_data.r#move.promotion();
        let color = self.active_color.inverse();

        let piece_at_to: Piece;

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
            let mut ep_mask = Bitboard::EMPTY;

            let is_en_passant = match move_data.flags.en_passant_file() {
                Some(file) => {
                    let rank = color.inverse().en_passant_rank();
                    ep_mask |= Bitboard(1 << (rank * 8 + file));

                    moved_piece == Piece::Pawn
                        && captured_piece == Piece::Pawn
                        && ep_mask == to.bitboard()
                }
                None => false,
            };

            let square = if is_en_passant {
                let mut shifted_mask = ep_mask;
                match color {
                    Color::White => shifted_mask >>= 8,
                    Color::Black => shifted_mask <<= 8,
                }

                Square::ALL[shifted_mask.0.trailing_zeros() as usize]
            } else {
                to
            };

            self.add_piece(captured_piece, color.inverse(), square);
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
    fn default() -> Self {
        let mut board = Board::new();
        board.load_from_fen(START_FEN).unwrap();
        board
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[rustfmt::skip]
        const PIECE_CHARS: [char; 12] = [
            'n', 'b', 'r', 'q', 'p', 'k',
            'N', 'B', 'R', 'Q', 'P', 'K',
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

        for (i, bb) in self.piece_bitboards.into_iter().enumerate() {
            let piece_char = PIECE_CHARS[i];

            for square in bb.active() {
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
