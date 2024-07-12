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
#[derive(Debug)]
pub struct SearchManager {
    handles: Vec<JoinHandle<()>>,
    pub running: bool,

    // Shared data
    pub move_gen: Arc<MoveGen>,
    pub cancelled: Arc<Mutex<AtomicBool>>,
    pub best_move: Arc<Mutex<Move>>,
    pub best_eval: Arc<Mutex<AtomicI32>>,
}

impl SearchManager {
    pub fn new(move_gen: Arc<MoveGen>) -> Self {
        Self {
            handles: Vec::new(),
            running: false,

            move_gen,
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

        self.running = true;
    }

    pub fn cancel(&mut self) {
        self.cancelled
            .lock()
            .unwrap()
            .store(true, Ordering::Relaxed);

        self.running = false;

        for _ in 0..self.handles.len() {
            self.handles.pop().unwrap().join().unwrap();
        }
    }

    pub fn best_move(&self) -> Move {
        *self.best_move.lock().unwrap()
    }

    pub fn best_eval(&self) -> i32 {
        self.best_eval.lock().unwrap().load(Ordering::Relaxed)
    }
}

/// Represents a single thread performing a search
#[derive(Debug, Clone)]
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
            self.alpha_beta(0, -999999, 999999, i);

            if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
                break;
            }

            println!("{i}: {}", self.best_move_so_far);

            *self.best_move.lock().unwrap() = self.best_move_so_far;
            self.best_eval
                .lock()
                .unwrap()
                .store(self.best_eval_so_far, Ordering::Relaxed);

            i += 1;
        }
    }

    fn alpha_beta(&mut self, ply_from_root: u8, mut alpha: i32, beta: i32, depth: u8) -> i32 {
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
            let score = -self.alpha_beta(ply_from_root + 1, -beta, -alpha, depth - 1);
            self.board.unmake_move(move_data).unwrap();

            if self.cancelled.lock().unwrap().load(Ordering::Relaxed) {
                break;
            }

            if score >= beta {
                return beta;
            }

            if score > alpha {
                if ply_from_root == 0 {
                    self.best_move_so_far = mv;
                    self.best_eval_so_far = score;
                }
                alpha = score;
            }
        }

        alpha
    }
}
