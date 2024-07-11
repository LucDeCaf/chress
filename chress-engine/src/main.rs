use std::{
    io::{self, BufRead},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use chress::board::Board;

use chress_engine::search::SearchManager;

extern crate chress;

fn main() -> std::io::Result<()> {
    // Threading
    let cancelled = Arc::new(Mutex::new(AtomicBool::new(false)));

    let board = Board::default();

    let mut search_manager = SearchManager::new();

    let mut buf = String::new();
    let mut stdin = io::stdin().lock();

    loop {
        // Get input
        buf.clear();
        if let Err(_) = stdin.read_line(&mut buf) {
            continue;
        }
        buf = buf.trim().to_owned();

        // Parse input into command string
        let mut parts = buf.split(' ');

        let command = parts.next().unwrap();
        let _arguments: Vec<&str> = parts.collect();

        match command {
            "quit" => {
                break;
            }

            "go" => {
                search_manager.start_search(board);
            }

            "stop" => {
                // Cancel the current search
                cancelled.lock().unwrap().store(true, Ordering::Relaxed);

                // Write the best move
                println!("bestmove {}", search_manager.best_move());
            }

            _ => continue,
        }
    }

    Ok(())
}
