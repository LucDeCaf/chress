use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use chress::{board::Board, r#move::Move};

use crate::evaluation::evaluate;

pub struct Searcher {
    board: Board,

    best_move_so_far: Move,
    best_eval_so_far: i32,
    best_move: Move,
    best_eval: i32,

    cancelled: Arc<Mutex<AtomicBool>>,

    pub handles: Vec<JoinHandle<()>>,
}

impl Searcher {
    const MAX_SCORE: i32 = 999999;
    const MIN_SCORE: i32 = -999999;

    pub const ALPHA: i32 = Self::MAX_SCORE;
    pub const BETA: i32 = Self::MIN_SCORE;

    pub fn new(board: Board, cancelled: Arc<Mutex<AtomicBool>>) -> Self {
        Self {
            board,

            best_move_so_far: Move::NULLMOVE,
            best_eval_so_far: 0,
            best_move: Move::NULLMOVE,
            best_eval: 0,

            cancelled,

            handles: Vec::new(),
        }
    }

    pub fn start_search(&mut self) {
        // Reset data from previous search
        self.best_move_so_far = Move::NULLMOVE;
        self.best_move = Move::NULLMOVE;
        self.best_eval_so_far = 0;
        self.best_eval = 0;

        // Start search
        self.start_iterative_deepening();

        // If search was cancelled before any moves were looked at
        if self.best_move == Move::NULLMOVE {
            self.best_move = self.board.legal_moves()[0];
        }

        // Reset 'cancelled'
        self.cancelled
            .lock()
            .unwrap()
            .store(false, Ordering::Relaxed);
    }

    pub fn best_move(&self) -> (Move, i32) {
        (self.best_move, self.best_eval)
    }

    pub fn start_iterative_deepening(&mut self) {
        let mut i = 1;

        while i < 100 {
            self.search(i, Self::ALPHA, Self::BETA);

            self.best_move = self.best_move_so_far;
            self.best_eval = self.best_eval_so_far;

            if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
                break;
            }

            i += 1;
        }
    }

    pub fn search(&mut self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
            return 0;
        }

        if depth == 0 {
            return evaluate(&self.board);
        }

        let moves = self.board.legal_moves();

        let mut eval;

        for mv in moves {
            self.board.make_move(mv).unwrap();

            eval = -self.search(depth - 1, -beta, -alpha);

            self.board.unmake_move().unwrap();

            if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
                break;
            }

            if eval >= beta {
                return beta;
            }

            if eval > alpha {
                // New best move found!
                self.best_move_so_far = mv;
                self.best_eval_so_far = eval;

                alpha = eval;
            }
        }

        alpha
    }
}
