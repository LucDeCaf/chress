use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

use chress::{board::Board, r#move::Move};

pub fn perft(board: &mut Board, depth: usize) {
    let (chress_total, chress_move_results) = board.divide(depth);

    let mut found_moves = HashMap::new();
    for mv in chress_move_results {
        found_moves.insert(mv.0, mv.1);
    }

    // * Get stockfish perft results
    let mut stockfish = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = stockfish.stdin.take().unwrap();
    let stdout = stockfish.stdout.take().unwrap();

    // Setup position
    stdin
        .write_all(format!("position fen {}\n", board.fen()).as_bytes())
        .unwrap();

    // Run perft
    stdin
        .write_all(format!("go perft {depth}\n").as_bytes())
        .unwrap();

    let mut reader = BufReader::new(stdout);
    let mut buf = Vec::new();

    // First three lines are info lines, so ignore them
    reader.read_until(b'\n', &mut buf).unwrap();
    println!("{}", String::from_utf8_lossy(&buf));
    reader.read_until(b'\n', &mut buf).unwrap();
    println!("{}", String::from_utf8_lossy(&buf));
    reader.read_until(b'\n', &mut buf).unwrap();
    println!("{}", String::from_utf8_lossy(&buf));
    buf.clear();

    // Read until letter 'N' appears (can't appear until last line in 'Nodes searched: {nodes}')
    reader.read_until(b'N', &mut buf).unwrap();
    // Read the last line
    reader.read_until(b'\n', &mut buf).unwrap();

    // Quit stockfish
    stdin.write_all(b"quit\n").unwrap();
    stockfish.wait().unwrap();

    let output = String::from_utf8_lossy(&buf).to_string();

    let mut expected_moves = HashMap::new();
    let mut expected_total: u64 = 0;

    let mut set_total = false;

    // Extract output
    for line in output.lines() {
        let line = line.trim();

        if set_total {
            expected_total = line.split(' ').last().unwrap().parse().unwrap();
            break;
        }

        if line.is_empty() {
            set_total = true;
            continue;
        }

        // Extract move from line
        let mut parts = line.split(':');

        let r#move = Move::try_from(parts.next().unwrap()).unwrap();
        let count: u64 = parts.next().unwrap().trim().parse().unwrap();

        expected_moves.insert(r#move, count);
    }

    let mut found = HashSet::new();
    let mut expected = HashSet::new();

    for mv in found_moves.iter() {
        found.insert(*mv.0);
    }

    for mv in expected_moves.iter() {
        expected.insert(*mv.0);
    }

    println!("---- COMPARISON RESULTS ----");
    println!();

    let matching = found.intersection(&expected);
    let extra = found.difference(&expected);
    let missing = expected.difference(&found);

    println!("Move\tExpect\tFound");
    for matching in matching {
        let found_val = found_moves.get(matching).unwrap();
        let expected_val = expected_moves.get(matching).unwrap();

        let mismatch_char = match found_val.cmp(expected_val) {
            Ordering::Greater => '>',
            Ordering::Less => '<',
            Ordering::Equal => ' ',
        };

        println!(
            "{}\t{}\t{}\t{}",
            matching, expected_val, found_val, mismatch_char
        );
    }

    println!();

    println!("Missing moves:");
    for missing in missing {
        println!("{missing}");
    }
    println!();

    println!("Unexpected moves:");
    for extra in extra {
        println!("{extra}");
    }
    println!();

    println!(
        "Total node difference = {} - {} = {}",
        chress_total,
        expected_total,
        chress_total as i64 - expected_total as i64
    );
}
