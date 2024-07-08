#[cfg(test)]
pub mod perft_tests {
    use std::time;

    use chress::board::{Board, POSITION_2};

    #[test]
    fn perft_startpos() {
        let mut board = Board::default();

        let now = time::Instant::now();

        let perft = board.perft(6);

        let elapsed = now.elapsed();

        println!("{perft} nodes in {} seconds", elapsed.as_secs_f64());
        println!("{} nodes/second", perft as f64 / elapsed.as_secs_f64());
    }

    // ? Current speeds (1): ~853,000 nodes/sec
    // ? Current speeds (5): ~21,000,000 nodes/sec
    #[test]
    fn perft_kiwipete() {
        let mut board = Board::new();
        board.load_from_fen(POSITION_2).unwrap();

        let now = time::Instant::now();

        let perft = board.perft(5);

        let elapsed = now.elapsed();

        println!("{perft} nodes in {} seconds", elapsed.as_secs_f64());
        println!("{} nodes/second", perft as f64 / elapsed.as_secs_f64());
    }
}
