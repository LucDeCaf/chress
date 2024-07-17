pub const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
pub const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
pub const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
pub const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

#[cfg(test)]
pub mod perft_speed_tests {
    use chress::{board::Board, debug::perft, move_gen::MoveGen};
    use std::time;

    use super::*;

    #[test]
    fn startpos() {
        let move_gen = MoveGen::new();
        let board = Board::default();
        let depth = 4;

        let now = time::Instant::now();

        let perft = perft(board, &move_gen, depth);

        let elapsed = now.elapsed();

        println!("{perft} nodes in {} seconds", elapsed.as_secs_f64());
        println!("{} nodes/second", perft as f64 / elapsed.as_secs_f64());
    }

    // ? Current speeds (1): ~853,000 nodes/sec
    // ? Current speeds (5): ~21,000,000 nodes/sec
    // ?
    // ? Speed looks good enough to move onto the engine itself
    #[test]
    fn kiwipete() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(KIWIPETE, &move_gen).unwrap();
        let depth = 4;

        let now = time::Instant::now();

        let perft = perft(board, &move_gen, depth);

        let elapsed = now.elapsed();

        println!("{perft} nodes in {} seconds", elapsed.as_secs_f64());
        println!("{} nodes/second", perft as f64 / elapsed.as_secs_f64());
    }
}

#[cfg(test)]
pub mod perft_tests {
    use chress::{board::Board, debug::perft, move_gen::MoveGen};

    use super::*;

    #[test]
    fn startpos() {
        let move_gen = MoveGen::new();
        let board = Board::default();

        assert_eq!(perft(board, &move_gen, 5), 4865609);
    }

    #[test]
    fn kiwipete() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(KIWIPETE, &move_gen).unwrap();

        assert_eq!(perft(board, &move_gen, 5), 193690690);
    }

    #[test]
    fn position_3() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(POSITION_3, &move_gen).unwrap();

        assert_eq!(perft(board, &move_gen, 7), 178633661);
    }

    #[test]
    fn position_4() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(POSITION_4, &move_gen).unwrap();

        assert_eq!(perft(board, &move_gen, 5), 15833292);
    }

    #[test]
    fn position_5() {
        let move_gen = MoveGen::new();
        let board = Board::from_fen(POSITION_5, &move_gen).unwrap();

        assert_eq!(perft(board, &move_gen, 5), 89941194);
    }
}
