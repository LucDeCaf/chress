use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use chress::{
    board::{r#move::Move, Board},
    move_gen::MoveGen,
};

use crate::evaluation::evaluate;

/// Manages all searching threads and shared data
pub struct SearchManager {
    handles: Vec<JoinHandle<()>>,

    // Shared data
    pub move_gen: Arc<MoveGen>,
    pub cancelled: Arc<Mutex<AtomicBool>>,
    pub best_move: Arc<Mutex<Move>>,
    pub best_eval: Arc<Mutex<AtomicI32>>,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),

            move_gen: Arc::new(MoveGen::new()),
            cancelled: Arc::new(Mutex::new(AtomicBool::new(false))),
            best_move: Arc::new(Mutex::new(Move::NULLMOVE)),
            best_eval: Arc::new(Mutex::new(AtomicI32::new(0))),
        }
    }

    pub fn start_search(&mut self, position: Board) {
        // Reset data from prev search
        self.cancelled
            .lock()
            .unwrap()
            .store(false, Ordering::Relaxed);
        *self.best_move.lock().unwrap() = Move::NULLMOVE;

        // Clone shared data references
        let move_gen = Arc::clone(&self.move_gen);
        let cancelled = Arc::clone(&self.cancelled);
        let best_move = Arc::clone(&self.best_move);
        let best_eval = Arc::clone(&self.best_eval);

        // Start new search
        let new_search = Search::new(position, move_gen, cancelled, best_move, best_eval);
        self.handles.push(new_search.start());
    }

    pub fn cancel(&mut self) {
        self.cancelled
            .lock()
            .unwrap()
            .store(true, Ordering::Relaxed);
    }

    pub fn best_move(&self) -> Move {
        *self.best_move.lock().unwrap()
    }

    pub fn best_eval(&self) -> i32 {
        self.best_eval.lock().unwrap().load(Ordering::Relaxed)
    }
}

impl Default for SearchManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a single thread performing a search
pub struct Search {
    board: Board,
    best_move_so_far: Move,
    best_eval_so_far: i32,

    // Shared data
    move_gen: Arc<MoveGen>,
    cancelled: Arc<Mutex<AtomicBool>>,
    best_move: Arc<Mutex<Move>>,
    best_eval: Arc<Mutex<AtomicI32>>,
}

impl Search {
    const MAX_SCORE: i32 = 999999;
    const MIN_SCORE: i32 = -999999;

    pub const ALPHA: i32 = Self::MAX_SCORE;
    pub const BETA: i32 = Self::MIN_SCORE;

    pub fn new(
        board: Board,
        move_gen: Arc<MoveGen>,
        cancelled: Arc<Mutex<AtomicBool>>,
        best_move: Arc<Mutex<Move>>,
        best_eval: Arc<Mutex<AtomicI32>>,
    ) -> Self {
        Self {
            board,
            best_move_so_far: Move::NULLMOVE,
            best_eval_so_far: 0,

            // Shared data
            move_gen,
            cancelled,
            best_move,
            best_eval,
        }
    }

    pub fn start(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.start_iterative_deepening())
    }

    fn start_iterative_deepening(&mut self) {
        let mut i = 1;

        while i < 254 {
            self.alpha_beta(Self::BETA, Self::ALPHA, i);

            *self.best_move.lock().unwrap() = self.best_move_so_far;
            self.best_eval
                .lock()
                .unwrap()
                .store(self.best_eval_so_far, Ordering::Relaxed);

            if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
                break;
            }

            i += 1;
        }
    }

    fn alpha_beta(&mut self, mut alpha: i32, beta: i32, depth: u8) -> i32 {
        if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
            return 0;
        }

        if depth == 0 {
            return evaluate(&self.board);
        }

        let mut moves = Vec::new();
        self.move_gen.legal_moves(&self.board, &mut moves);

        for mv in moves {
            let move_data = self.board.make_move(mv).unwrap();
            let score = -self.alpha_beta(-beta, -alpha, depth - 1);
            // println!("{mv}: {score}");
            self.board.unmake_move(move_data).unwrap();

            if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
                break;
            }

            if score >= beta {
                return beta;
            }

            if score > alpha {
                self.best_move_so_far = mv;
                self.best_eval_so_far = score;
                alpha = score;
            }
        }

        *self.best_move.lock().unwrap() = self.best_move_so_far;
        self.best_eval
            .lock()
            .unwrap()
            .store(self.best_eval_so_far, Ordering::Relaxed);

        alpha
    }
}
