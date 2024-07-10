use std::{
    io::{self, BufRead},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use chress::board::Board;
use chress_engine::search::searcher::Searcher;

extern crate chress;

fn main() -> std::io::Result<()> {
    // Threading
    let cancelled = Arc::new(Mutex::new(AtomicBool::new(false)));

    let board = Board::default();

    let mut searcher = Searcher::new(board, Arc::clone(&cancelled));

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
        let arguments: Vec<&str> = parts.collect();

        match command {
            "quit" => {
                break;
            }

            "go" => {
                searcher.start_search();
            }

            "stop" => {
                // Cancel the current search
                cancelled.lock().unwrap().store(true, Ordering::Relaxed);
            }

            _ => continue,
        }
    }

    // Just here for safety
    for handle in searcher.handles {
        handle.join().unwrap();
    }

    Ok(())
}
