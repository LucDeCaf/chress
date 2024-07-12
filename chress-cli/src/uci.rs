use std::{io::stdin, sync::Arc};

use chress::{
    board::{r#move::Move, Board},
    move_gen::MoveGen,
};
use chress_engine::search::{MoveTime, SearchManager, SearchSettings};

const UCI_STRING: &str = "id name Chress\nid author Luc de Cafmeyer\nuciok";

pub fn uci() -> std::io::Result<()> {
    let mut board = Board::default();
    let move_gen = Arc::new(MoveGen::new());
    let mut search_manager = SearchManager::new(Arc::clone(&move_gen));

    let mut buf = String::new();
    let mut arguments: Vec<String> = Vec::new();

    println!("{}", UCI_STRING);

    loop {
        stdin().read_line(&mut buf)?;

        let mut input = buf.split_ascii_whitespace().map(|i| String::from(i.trim()));

        let Some(command) = input.next() else {
            continue;
        };

        arguments.extend(input);

        match command.as_str() {
            "quit" => break,

            "uci" => println!("{}", UCI_STRING),
            "ucinewgame" => println!("readyok"),
            "isready" => println!("readyok"),

            "position" => {
                let Some(first) = arguments.first() else {
                    continue;
                };

                let first_move_index = match first.as_str() {
                    "startpos" => {
                        board = Board::default();
                        1
                    }
                    "fen" => {
                        let Some(fen) = arguments.get(1..7) else {
                            continue;
                        };

                        let fen = fen.join(" ");

                        board.load_from_fen(&fen, &move_gen).unwrap();

                        7
                    }
                    _ => panic!("Invalid arguments for position"),
                };

                let moves = arguments
                    .get(first_move_index..)
                    .expect("Invalid arguments for position");

                for mv in moves {
                    let mv = Move::try_from(mv.as_str()).expect("Bad format for UCI move");

                    board.make_move(mv).unwrap();
                }
            }

            "go" => {
                let mut settings = SearchSettings::default();

                for (i, arg) in arguments.iter().enumerate() {
                    match arg.as_str() {
                        "infinite" => settings.movetime = MoveTime::Infinite,
                        "movetime" => {
                            let millis = arguments
                                .get(i + 1)
                                .expect("Missing argument for movetime")
                                .parse::<u32>()
                                .expect("Invalid argument for movetime");
                            settings.movetime = MoveTime::Millis(millis);
                        }
                        _ => (),
                    }
                }

                search_manager.settings = settings;

                search_manager.start_search(board);
            }

            "stop" => {
                if !search_manager.running {
                    continue;
                }

                search_manager.stop();
            }

            _ => (),
        }

        arguments.clear();
        buf.clear();
    }

    Ok(())
}
