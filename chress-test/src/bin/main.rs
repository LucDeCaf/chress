use std::{
    io::{BufRead, Write},
    path::Path,
};

use chress::board::{r#move::Move, Board};
use chress_test::Engine;

// Note: Programs run in the workspace directory
fn main() -> std::io::Result<()> {
    let mut engine_1 =
        Engine::new("engine1".to_owned(), Path::new("target/release/chress_cli")).unwrap();

    let mut engine_2 =
        Engine::new("engine2".to_owned(), Path::new("target/release/chress_cli")).unwrap();

    let mut board = Board::default();
    let mut moves: Vec<Move> = Vec::new();
    let mut position_string = String::from("position startpos \n");

    let mut buf = String::new();

    // Engine setup
    engine_1.stdin.write_all(b"uci\nucinewgame\nisready\n")?;
    engine_2.stdin.write_all(b"uci\nucinewgame\nisready\n")?;

    // Game loop
    loop {
        buf.clear();

        let engine = if moves.len() % 2 == 0 {
            &mut engine_1
        } else {
            &mut engine_2
        };

        engine.stdin.write_all(position_string.as_bytes())?;
        engine.stdin.write_all(b"go movetime 500\n")?;

        engine.stdout.read_line(&mut buf)?;

        let mv = Move::try_from(buf.as_ref()).unwrap();

        position_string.push_str(buf.as_ref());
        moves.push(mv);

        // TODO: Implement game-end logic

        // ?  Game ends are as follows:
        // ?  - Stalemate
        // ?  - Checkmate
        // ?  - Insufficient material

        if let Some(game_end) = board.game_over() {
            break;
        }
    }

    Ok(())
}
