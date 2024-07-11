use crate::{
    board::{
        bitboard::Bitboard,
        color::Color,
        piece::Piece,
        r#move::Move,
        sliding_moves::{create_bishop_table, create_rook_table, magic_index},
        square::Square,
        Board, CASTLING_BLOCKERS, CASTLING_CHECKABLES, CASTLING_DESTINATIONS,
        KING_STARTING_SQUARES,
    },
    build::{
        magics::{BISHOP_MAGICS, ROOK_MAGICS},
        movemasks::{KING_MOVES, KNIGHT_MOVES, PAWN_CAPTURES},
    },
};

// Not deriving Copy because even Cloning this struct would be a bad idea
#[derive(Debug, Clone)]
pub struct MoveGen {
    rook_table: Vec<Bitboard>,
    bishop_table: Vec<Bitboard>,
}
impl MoveGen {
    pub fn new() -> Self {
        Self {
            rook_table: create_rook_table(),
            bishop_table: create_bishop_table(),
        }
    }

    // * The next few private functions are implemented on MoveGen because I don't really
    // * want to expose them via a public API on the board, but still need to access
    // * them from MoveGen
    fn white_pawns_able_to_push(board: &Board, empty: Bitboard) -> Bitboard {
        (empty >> 8) & board.bitboard(Piece::Pawn, Color::White)
    }

    fn black_pawns_able_to_push(board: &Board, empty: Bitboard) -> Bitboard {
        (empty << 8) & board.bitboard(Piece::Pawn, Color::Black)
    }

    fn white_pawns_able_to_double_push(board: &Board, empty: Bitboard) -> Bitboard {
        const RANK_4: Bitboard = Bitboard(0x00000000FF000000);
        let empty_in_rank_3 = ((empty & RANK_4) >> 8) & empty;
        Self::white_pawns_able_to_push(board, empty_in_rank_3)
    }

    fn black_pawns_able_to_double_push(board: &Board, empty: Bitboard) -> Bitboard {
        const RANK_5: Bitboard = Bitboard(0x000000FF00000000);
        let empty_in_rank_6 = ((empty & RANK_5) << 8) & empty;
        Self::black_pawns_able_to_push(board, empty_in_rank_6)
    }

    /// Used with sliding pieces
    fn append_moves_getter(
        &self,
        board: &Board,
        moves: &mut Vec<Move>,
        mut pieces: Bitboard,
        move_getter: fn(&Self, &Board, Square) -> Bitboard,
    ) {
        for _ in 0..pieces.0.count_ones() {
            let i = pieces.pop_lsb();

            let from = Square::ALL[i as usize];
            let mut targets = move_getter(self, board, from);

            for _ in 0..targets.0.count_ones() {
                let j = targets.pop_lsb();
                let to = Square::ALL[j as usize];

                moves.push(Move::new(from, to));
            }
        }
    }

    /// Used with non-sliding pieces as it showed significant performance gains
    fn append_moves_table(
        &self,
        moves: &mut Vec<Move>,
        mut pieces: Bitboard,
        friendly_pieces: Bitboard,
        move_table: &[Bitboard; 64],
    ) {
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

    pub fn rook_attacks(&self, square: Square, blockers: Bitboard) -> Bitboard {
        self.rook_table[magic_index(&ROOK_MAGICS[square as usize], blockers)]
    }

    pub fn bishop_attacks(&self, square: Square, blockers: Bitboard) -> Bitboard {
        self.bishop_table[magic_index(&BISHOP_MAGICS[square as usize], blockers)]
    }

    pub fn queen_attacks(&self, square: Square, blockers: Bitboard) -> Bitboard {
        self.rook_attacks(square, blockers) | self.bishop_attacks(square, blockers)
    }

    pub fn pseudo_rook_moves(&self, board: &Board, square: Square) -> Bitboard {
        let friendly_pieces = board.friendly_pieces();
        let enemy_pieces = board.enemy_pieces();

        self.rook_attacks(square, friendly_pieces | enemy_pieces) & !friendly_pieces
    }

    pub fn pseudo_bishop_moves(&self, board: &Board, square: Square) -> Bitboard {
        let friendly = board.friendly_pieces();
        let enemy = board.enemy_pieces();

        self.bishop_attacks(square, friendly | enemy) & !friendly
    }

    pub fn pseudo_queen_moves(&self, board: &Board, square: Square) -> Bitboard {
        let friendly_pieces = board.friendly_pieces();
        let enemy_pieces = board.enemy_pieces();
        let blockers = friendly_pieces | enemy_pieces;

        let attacks = self.rook_attacks(square, blockers) | self.bishop_attacks(square, blockers);

        attacks & !friendly_pieces
    }

    // ? This function has been benchmarked against the branchless version, which was slower.
    /// Checks if a square is seen by pieces of a certain color for the
    /// purpose of legal move generation
    pub fn square_attacked_by(&self, board: &Board, square: Square, attacker_color: Color) -> bool {
        let pawn_attackers = PAWN_CAPTURES[attacker_color.inverse() as usize][square as usize]
            & board.bitboard(Piece::Pawn, attacker_color);

        if !pawn_attackers.is_empty() {
            return true;
        }

        let king_attacks =
            KING_MOVES[square as usize] & board.bitboard(Piece::King, attacker_color);

        if !king_attacks.is_empty() {
            return true;
        }

        let knight_attacks =
            KNIGHT_MOVES[square as usize] & board.bitboard(Piece::Knight, attacker_color);

        if !knight_attacks.is_empty() {
            return true;
        }

        let rook_attacks = self.rook_attacks(square, board.occupied());
        let rooks_queens = board.bitboard(Piece::Rook, attacker_color)
            | board.bitboard(Piece::Queen, attacker_color);

        if !(rook_attacks & rooks_queens).is_empty() {
            return true;
        }

        let bishop_attacks = self.bishop_attacks(square, board.occupied());
        let bishops_queens = board.bitboard(Piece::Bishop, attacker_color)
            | board.bitboard(Piece::Queen, attacker_color);

        if !(bishop_attacks & bishops_queens).is_empty() {
            return true;
        }

        false
    }

    /// Get all pseudolegal moves
    pub fn pseudolegal_moves(&self, board: &Board, moves: &mut Vec<Move>) {
        let color = board.active_color;
        let attacker_color = color.inverse();

        let friendly_pieces = board.friendly_pieces();
        let enemy_pieces = board.enemy_pieces();

        let all_pieces = friendly_pieces | enemy_pieces;
        let empty_squares = !all_pieces;

        // Pawn move data
        let pawn_data = [
            (
                Self::black_pawns_able_to_push(board, empty_squares),
                Self::black_pawns_able_to_double_push(board, empty_squares),
            ),
            (
                Self::white_pawns_able_to_push(board, empty_squares),
                Self::white_pawns_able_to_double_push(board, empty_squares),
            ),
        ];

        let (mut single_push_froms, mut double_push_froms) = pawn_data[color as usize];

        let mut pawns = board.bitboard(Piece::Pawn, color);

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
        let file = board.flags.en_passant_file_unchecked();

        let ep_square = Square::ALL[(rank * 8 + file) as usize];
        let can_en_passant = board.flags.en_passant_valid();

        let reset_mask = Bitboard::UNIVERSE * can_en_passant;

        let pawns_that_can_take = PAWN_CAPTURES[color.inverse() as usize][ep_square as usize]
            & board.bitboard(Piece::Pawn, color);

        let mut actual_pawns = pawns_that_can_take & reset_mask;

        // Add pseudolegal en passant moves
        for _ in 0..actual_pawns.0.count_ones() {
            let from = Square::ALL[actual_pawns.pop_lsb() as usize];

            moves.push(Move::new(from, ep_square));
        }

        // King moves
        let king_index = board.bitboard(Piece::King, color).0.trailing_zeros() as usize;
        let king_square = Square::ALL[king_index];

        // let mut targets = self.pseudo_king_moves(board, king_square);
        let mut targets = KING_MOVES[king_square as usize] & !friendly_pieces;
        targets.append_moves_from(moves, king_square);

        // Castling
        // Check if king is on start square and not in check
        let king_start_square = KING_STARTING_SQUARES[color as usize];
        let on_start_square = king_square == king_start_square;
        let in_check = self.square_attacked_by(board, king_start_square, attacker_color);

        if on_start_square && !in_check {
            let blocker_list = CASTLING_BLOCKERS[color as usize];
            let targets = CASTLING_DESTINATIONS[color as usize];
            let allowed = [board.flags.kingside(color), board.flags.queenside(color)];

            let occupied = board.occupied();

            'outer: for i in 0..2 {
                // Disallow castling if it is disallowed (omg so smart)
                if !allowed[i] {
                    continue;
                }

                let blockers = blocker_list[i];

                // Check for pieces in the way
                if !(blockers & occupied).is_empty() {
                    continue;
                }

                // Check if castling through/out of check
                // Don't need to check if castling into check as that is checked
                // in legal_moves already (would be redundant)
                let mut checkables = CASTLING_CHECKABLES[color as usize][i];

                for _ in 0..checkables.0.count_ones() {
                    let square = Square::ALL[checkables.pop_lsb() as usize];

                    if self.square_attacked_by(board, square, attacker_color) {
                        continue 'outer;
                    }
                }

                // Add castling as pseudolegal move
                moves.push(Move::new(king_start_square, targets[i]));
            }
        }

        // Knight moves
        let knights = board.bitboard(Piece::Knight, color);
        self.append_moves_table(moves, knights, friendly_pieces, &KNIGHT_MOVES);

        // Rook moves
        let rooks = board.bitboard(Piece::Rook, color);
        self.append_moves_getter(board, moves, rooks, Self::pseudo_rook_moves);

        // Bishop moves
        let bishops = board.bitboard(Piece::Bishop, color);
        self.append_moves_getter(board, moves, bishops, Self::pseudo_bishop_moves);

        // Queen moves
        let queens = board.bitboard(Piece::Queen, color);
        self.append_moves_getter(board, moves, queens, Self::pseudo_queen_moves);
    }

    /// Takes a mutable reference to self because to check legality the
    /// move is made and unmade on the board before checking if the king
    /// is under attack.
    pub fn is_legal_move(&self, mut board: Board, r#move: Move) -> bool {
        let current_color = board.active_color;
        let attacker_color = current_color.inverse();

        board.make_move(r#move).unwrap();

        let king_square = Square::ALL[board
            .bitboard(Piece::King, current_color)
            .0
            .trailing_zeros() as usize];

        let is_legal = !self.square_attacked_by(&board, king_square, attacker_color);

        is_legal
    }

    /// Generate all legal moves at the current position
    pub fn legal_moves(&self, board: &Board, moves: &mut Vec<Move>) {
        self.pseudolegal_moves(board, moves);

        let mut i = 0;
        let mut len = moves.len();

        while i < len {
            let mv = moves[i];

            if !self.is_legal_move(board.clone(), mv) {
                moves.swap_remove(i);
                len -= 1;
            } else {
                i += 1;
            }
        }
    }
}

impl Default for MoveGen {
    fn default() -> Self {
        Self::new()
    }
}
