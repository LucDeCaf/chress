#[cfg(test)]
pub mod perft_tests {
    use std::time;

    use chress::board::Board;

    pub const KIWIPETE: &str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    pub const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    pub const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    pub const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

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
    // ?
    // ? Speed looks good enough to move onto the engine itself
    #[test]
    fn perft_kiwipete() {
        let mut board = Board::from_fen(KIWIPETE).unwrap();

        let now = time::Instant::now();

        let perft = board.perft(5);

        let elapsed = now.elapsed();

        println!("{perft} nodes in {} seconds", elapsed.as_secs_f64());
        println!("{} nodes/second", perft as f64 / elapsed.as_secs_f64());
    }
}
