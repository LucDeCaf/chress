use std::{
    error::Error,
    fmt::Display,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    bitboard::Bitboard,
    color::Color,
    flags::Flags,
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
pub const POSITION_2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
pub const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
pub const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

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
    table[Square::A1 as usize] = Flags(!Flags::WHITE_QUEENSIDE.0);
    table[Square::A8 as usize] = Flags(!Flags::BLACK_QUEENSIDE.0);
    table[Square::H1 as usize] = Flags(!Flags::WHITE_KINGSIDE.0);
    table[Square::H8 as usize] = Flags(!Flags::BLACK_KINGSIDE.0);
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
    InvalidPosition,
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
    fn new() -> Self {
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

    pub fn from_fen(fen: &str) -> Result<Self, ParseFenError> {
        let mut board = Board::new();
        board.load_from_fen(fen)?;
        Ok(board)
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
        if self.square_attacked_by(enemy_king_square, self.active_color) {
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

        let moves = self.pseudolegal_moves();

        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            return moves
                .into_iter()
                .filter(|mv| self.is_legal_move(*mv))
                .count() as u64;
        }

        for r#move in moves {
            self.make_move(r#move).unwrap();

            let king_square = Square::ALL[self
                .bitboard(Piece::King, self.active_color.inverse())
                .0
                .trailing_zeros() as usize];

            let in_check = self.square_attacked_by(king_square, self.active_color);

            if !in_check {
                results += self.perft(depth - 1);
            }

            self.unmake_move().unwrap();
        }

        results
    }

    pub fn divide(&mut self, depth: usize) -> (u64, Vec<(Move, u64)>) {
        let mut total = 0;
        let moves = self.pseudolegal_moves();
        let mut results = Vec::with_capacity(moves.len());

        for r#move in moves {
            self.make_move(r#move).unwrap();
            let king_square = Square::ALL[self
                .bitboard(Piece::King, self.active_color.inverse())
                .0
                .trailing_zeros() as usize];

            let in_check = self.square_attacked_by(king_square, self.active_color);

            if !in_check {
                let count = self.perft(depth - 1);
                total += count;
                results.push((r#move, count));
            }

            self.unmake_move().unwrap();
        }

        results.sort_by(|(a, _), (b, _)| a.cmp(b));

        (total, results)
    }

    pub fn perft_parallel(&mut self, depth: usize) -> u64 {
        let results = Arc::new(Mutex::new(AtomicU64::new(0)));
        let moves = self.pseudolegal_moves();

        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            return moves
                .into_iter()
                .filter(|mv| self.is_legal_move(*mv))
                .count() as u64;
        }

        let mut handles = Vec::new();

        for r#move in moves {
            self.make_move(r#move).unwrap();

            let king_square = Square::ALL[self
                .bitboard(Piece::King, self.active_color.inverse())
                .0
                .trailing_zeros() as usize];

            let in_check = self.square_attacked_by(king_square, self.active_color);

            if !in_check {
                let cloned_board = self.clone();
                let results = Arc::clone(&results);

                handles.push(thread::spawn(move || {
                    let mut board = cloned_board;

                    let perft = board.perft(depth - 1);

                    let results = results.lock().unwrap();
                    results.fetch_add(perft, Ordering::Relaxed);
                }));
            }

            self.unmake_move().unwrap();
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let results = results.lock().unwrap();
        results.load(Ordering::Relaxed)
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
        self.white_pieces() | self.black_pieces()
    }

    pub fn white_pieces(&self) -> Bitboard {
        self.piece_bitboards[6]
            | self.piece_bitboards[7]
            | self.piece_bitboards[8]
            | self.piece_bitboards[9]
            | self.piece_bitboards[10]
            | self.piece_bitboards[11]
    }

    pub fn black_pieces(&self) -> Bitboard {
        self.piece_bitboards[0]
            | self.piece_bitboards[1]
            | self.piece_bitboards[2]
            | self.piece_bitboards[3]
            | self.piece_bitboards[4]
            | self.piece_bitboards[5]
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

    pub fn friendly_pieces(&self) -> Bitboard {
        let off = self.active_color as usize * 6;

        self.piece_bitboards[off]
            | self.piece_bitboards[off + 1]
            | self.piece_bitboards[off + 2]
            | self.piece_bitboards[off + 3]
            | self.piece_bitboards[off + 4]
            | self.piece_bitboards[off + 5]
    }

    pub fn enemy_pieces(&self) -> Bitboard {
        let off = self.active_color as usize * 6;

        self.piece_bitboards[6 - off]
            | self.piece_bitboards[7 - off]
            | self.piece_bitboards[8 - off]
            | self.piece_bitboards[9 - off]
            | self.piece_bitboards[10 - off]
            | self.piece_bitboards[11 - off]
    }

    pub fn empty(&self) -> Bitboard {
        !self.occupied()
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
        KNIGHT_MOVES[square as usize] & !self.friendly_pieces()
    }

    pub fn king_moves(&self, square: Square) -> Bitboard {
        KING_MOVES[square as usize] & !self.friendly_pieces()
    }

    /// Used with sliding pieces
    pub fn append_moves_getter(
        &self,
        moves: &mut Vec<Move>,
        mut pieces: Bitboard,
        move_getter: fn(&Self, Square) -> Bitboard,
    ) {
        for _ in 0..pieces.0.count_ones() {
            let i = pieces.pop_lsb();

            let from = Square::ALL[i as usize];
            let mut targets = move_getter(self, from);

            for _ in 0..targets.0.count_ones() {
                let j = targets.pop_lsb();
                let to = Square::ALL[j as usize];

                moves.push(Move::new(from, to));
            }
        }
    }

    /// Used with non-sliding pieces as it showed significant performance gains
    pub fn append_moves_table(
        &self,
        moves: &mut Vec<Move>,
        mut pieces: Bitboard,
        move_table: &[Bitboard; 64],
    ) {
        let friendly_pieces = self.friendly_pieces();

        for _ in 0..pieces.0.count_ones() {
            let i = pieces.pop_lsb();

            let from = Square::ALL[i as usize];
            let mut targets = move_table[from as usize] & !friendly_pieces;

            for _ in 0..targets.0.count_ones() {
                let j = targets.pop_lsb();
                let to = Square::ALL[j as usize];

                moves.push(Move::new(from, to));
            }
        }
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
        let pawn_data = [
            (
                self.black_pawns_able_to_push(empty_squares),
                self.black_pawns_able_to_double_push(empty_squares),
            ),
            (
                self.white_pawns_able_to_push(empty_squares),
                self.white_pawns_able_to_double_push(empty_squares),
            ),
        ];

        let (mut single_push_froms, mut double_push_froms) = pawn_data[color as usize];

        let mut pawns = self.bitboard(Piece::Pawn, color);

        // Single moves
        for _ in 0..single_push_froms.0.count_ones() {
            let from = Square::ALL[single_push_froms.pop_lsb() as usize];

            let to_index = from as i8 + (8 * color.direction());
            let to = Square::try_from(to_index as usize).unwrap();

            // Promotion
            if to.rank() % 7 == 0 {
                // ? Not sure if this branch can actually be removed
                moves.push(Move::new_with_promotion(from, to, Piece::Knight));
                moves.push(Move::new_with_promotion(from, to, Piece::Bishop));
                moves.push(Move::new_with_promotion(from, to, Piece::Rook));
                moves.push(Move::new_with_promotion(from, to, Piece::Queen));
            } else {
                moves.push(Move::new(from, to));
            }
        }

        // Double moves
        for _ in 0..double_push_froms.0.count_ones() {
            let from = Square::ALL[double_push_froms.pop_lsb() as usize];

            let to_index = from as isize + (16 * color.direction()) as isize;
            let to = Square::ALL[to_index as usize];

            moves.push(Move::new(from, to));
        }

        // Captures
        for _ in 0..pawns.0.count_ones() {
            let from = Square::ALL[pawns.pop_lsb() as usize];

            let mut captures = PAWN_CAPTURES[color as usize][from as usize] & enemy_pieces;

            // Promotion
            for _ in 0..captures.0.count_ones() {
                let to = Square::ALL[captures.pop_lsb() as usize];

                // ? Not sure if this branch can actually be removed
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

        // Check for en passant
        let rank = color.inverse().en_passant_rank();
        let file = self.flags.en_passant_file_unchecked();

        let ep_square = Square::ALL[(rank * 8 + file) as usize];
        let can_en_passant = self.flags.en_passant_valid();

        let reset_mask = Bitboard::UNIVERSE * can_en_passant;

        let pawns_that_can_take = PAWN_CAPTURES[color.inverse() as usize][ep_square as usize]
            & self.bitboard(Piece::Pawn, color);

        let mut actual_pawns = pawns_that_can_take & reset_mask;

        // Add pseudolegal en passant moves
        for _ in 0..actual_pawns.0.count_ones() {
            let from = Square::ALL[actual_pawns.pop_lsb() as usize];

            moves.push(Move::new(from, ep_square));
        }

        // King moves
        let king_index = self.bitboard(Piece::King, color).0.trailing_zeros() as usize;
        let king_square = Square::ALL[king_index];

        let mut targets = self.king_moves(king_square);
        targets.append_moves_from(&mut moves, king_square);

        // Castling
        // Check if king is on start square and not in check
        let king_start_square = KING_STARTING_SQUARES[color as usize];
        let on_start_square = king_square == king_start_square;
        let in_check = self.square_attacked_by(king_start_square, attacker_color);

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
                let mut checkables = CASTLING_CHECKABLES[color as usize][i];

                for _ in 0..checkables.0.count_ones() {
                    let square = Square::ALL[checkables.pop_lsb() as usize];

                    if self.square_attacked_by(square, attacker_color) {
                        continue 'outer;
                    }
                }

                // Add castling as pseudolegal move
                moves.push(Move::new(king_start_square, targets[i]));
            }
        }

        // Knight moves
        let knights = self.bitboard(Piece::Knight, color);
        self.append_moves_table(&mut moves, knights, &KNIGHT_MOVES);

        // Rook moves
        let rooks = self.bitboard(Piece::Rook, color);
        self.append_moves_getter(&mut moves, rooks, Self::rook_moves);

        // Bishop moves
        let bishops = self.bitboard(Piece::Bishop, color);
        self.append_moves_getter(&mut moves, bishops, Self::bishop_moves);

        // Queen moves
        let queens = self.bitboard(Piece::Queen, color);
        self.append_moves_getter(&mut moves, queens, Self::queen_moves);

        moves
    }

    /// Gets all legal moves.
    ///
    /// Takes a mutable reference to self because to check legality each
    /// move is made and unmade on the board before checking if the king
    /// is under attack.
    pub fn legal_moves(&mut self) -> Vec<Move> {
        self.pseudolegal_moves()
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
            Square::ALL[self.bitboard(Piece::King, current_color).0.trailing_zeros() as usize];

        let is_legal = !self.square_attacked_by(king_square, attacker_color);

        self.unmake_move().unwrap();

        is_legal
    }

    // ? This function has been benchmarked against the branchless version, which was slower.
    /// Checks if a square is seen by pieces of a certain color for the
    /// purpose of legal move generation
    pub fn square_attacked_by(&self, square: Square, attacker_color: Color) -> bool {
        let pawn_attackers = PAWN_CAPTURES[attacker_color.inverse() as usize][square as usize]
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

    /// Plays a move on the board.
    ///
    /// This function will fail if the From square does not contain a piece.
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

        // Update move list
        self.move_list.push(move_data);

        Ok(())
    }

    // ! 4 branches, but they may be irreplaceable / too expensive to remove
    /// Unmakes a move on the board by popping the most recent move data off the stack.
    ///
    /// This function will fail if there is no piece to unmove on the To square, or if there
    /// is no data on the stack to pop.
    pub fn unmake_move(&mut self) -> Result<(), UnmakeMoveError> {
        let Some(move_data) = self.move_list.pop() else {
            return Err(UnmakeMoveError("no move to unmake".to_owned()));
        };

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
    ///
    /// ## Example
    /// ```
    /// use chress::board::{Board, START_FEN};
    ///
    /// // Create a board using 'default'
    /// let default_board = Board::default();
    ///
    /// // Create a board using 'new'
    /// let mut new_board = Board::new();
    /// new_board.load_from_fen(START_FEN).unwrap();
    ///
    /// assert_eq!(default_board, new_board);
    /// ```
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

        for (i, mut bb) in self.piece_bitboards.into_iter().enumerate() {
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
    extern crate test;

    use super::*;

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
        let board = Board::from_fen(POSITION_5).unwrap();

        assert_eq!(board.fen(), POSITION_5);
    }

    #[test]
    fn fen_en_passant() {
        const ONE_E4: &str = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

        let board = Board::from_fen(ONE_E4).unwrap();

        assert_eq!(board.fen(), ONE_E4);
    }
}
