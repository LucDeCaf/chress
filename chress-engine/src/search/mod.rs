use std::{
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chress::{
    board::{r#move::Move, Board},
    move_gen::MoveGen,
};

use crate::evaluation::evaluate;

#[derive(Debug, Clone, Copy)]
pub enum MoveTime {
    Infinite,
    Millis(u32),
}

impl Default for MoveTime {
    fn default() -> Self {
        Self::Infinite
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchSettings {
    pub ponder: bool,
    pub moves_to_go: Option<u16>,
    pub max_depth: Option<u8>,
    pub movetime: MoveTime,
}

/// Manages all searching threads and shared data
pub struct SearchManager {
    searches: Vec<JoinHandle<()>>,
    canceller: Option<Sender<bool>>,

    pub settings: SearchSettings,
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
            searches: Vec::new(),
            canceller: None,

            running: false,
            settings: SearchSettings::default(),

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
        self.best_eval.lock().unwrap().store(0, Ordering::Relaxed);

        // Activate canceller if search time is not infinite
        match self.settings.movetime {
            MoveTime::Millis(millis) => {
                // Create a new channel
                if let Some(tx) = &self.canceller {
                    let _ = tx.send(true);
                }

                let (tx, rx) = channel();
                self.canceller = Some(tx);

                let cancelled = Arc::clone(&self.cancelled);
                let best_move = Arc::clone(&self.best_move);
                let duration = Duration::from_millis(millis as u64);

                thread::spawn(move || {
                    // Wait for specified time
                    thread::sleep(duration);

                    // Prevent cancelling searches that shouldn't be cancelled.
                    //
                    // This scenario comes up when the engine is told to search for a specified
                    // amount of time but is later told to stop the search early. This creates a
                    // window of time during which no search is occuring but the canceller still wants
                    // to cancel any searches.
                    //
                    // If a new search is begun in this window, it is possible that the canceller
                    // will cancel the new search early and cause the engine to play a bad move.
                    //
                    // For this purpose, the engine will tell the canceller NOT to cancel when certain
                    // events occur, namely the search being stopped manually by the GUI.
                    if let Ok(dont_cancel) = rx.try_recv() {
                        if dont_cancel {
                            return;
                        }
                    }

                    cancelled.lock().unwrap().store(true, Ordering::Relaxed);
                    println!("bestmove {}", *best_move.lock().unwrap());
                });
            }
            _ => {
                self.canceller = None;
            }
        }

        // Clone shared data references
        let move_gen = Arc::clone(&self.move_gen);
        let cancelled = Arc::clone(&self.cancelled);
        let best_move = Arc::clone(&self.best_move);
        let best_eval = Arc::clone(&self.best_eval);

        // Start new search
        let new_search = Search::new(position, move_gen, cancelled, best_move, best_eval);
        self.searches.push(new_search.start());

        self.running = true;
    }

    pub fn stop(&mut self) {
        // Stop canceller from automatically cancelling
        if let Some(sender) = &self.canceller {
            let _ = sender.send(true);
        }

        self.canceller = None;

        self.cancelled
            .lock()
            .unwrap()
            .store(true, Ordering::Relaxed);

        self.running = false;

        for _ in 0..self.searches.len() {
            drop(self.searches.pop());
        }

        println!("bestmove {}", self.best_move());
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
