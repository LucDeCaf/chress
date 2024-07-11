extern crate chress;

use std::{io::stdin, process::Command};

use chress::{
    board::{r#move::Move, Board, START_FEN},
    move_gen::MoveGen,
};

use chress_cli::{perft, uci};

use chress_engine::search::searcher::SearchManager;

fn main() -> std::io::Result<()> {
    let mut board = Board::default();
    let move_gen = MoveGen::new();

    let mut input = String::new();

    let mut move_list = Vec::new();

    'main: loop {
        input.clear();
        stdin().read_line(&mut input)?;
        let input = input.trim();
        let commands = input.split(';');

        for command in commands {
            let input = command.trim();
            let mut iter = input.split_whitespace();

            let command = iter.next().unwrap().trim();
            let arguments = iter.map(|s| s.trim()).collect::<Vec<&str>>();

            match command {
                "startpos" => board.load_from_fen(START_FEN, &move_gen).unwrap(),
                "load" => {
                    if arguments[0] == "fen" {
                        let arguments = &arguments[1..];

                        if let Err(parse_error) =
                            board.load_from_fen(&arguments.join(" "), &move_gen)
                        {
                            println!("Error: {}", parse_error);
                        }
                    }
                }

                "fen" => {
                    println!("{}", board.fen());
                }

                "disp" | "display" | "d" => println!("{}\n", board),

                "clear" | "cls" => {
                    if cfg!(windows) {
                        Command::new("cls").spawn().expect("Failed to clear screen");
                    } else if cfg!(unix) {
                        Command::new("clear")
                            .spawn()
                            .expect("Failed to clear screen");
                    }
                }

                "undo" => {
                    if let Err(unmake_move_error) = board.unmake_move(move_list.pop().unwrap()) {
                        println!("Error: {}", unmake_move_error);
                    }
                }

                "moves" => {
                    let mut moves = Vec::new();

                    move_gen.legal_moves(&board, &mut moves);
                    moves.sort_unstable();

                    for r#move in moves {
                        println!("{move}");
                    }
                }

                "perft" => {
                    let Some(depth) = arguments.first().cloned() else {
                        println!("Missing arguments for perft");
                        break;
                    };

                    let Ok(depth) = depth.parse::<usize>() else {
                        println!("Invalid argument for perft: '{}'", arguments[0]);
                        break;
                    };

                    perft::perft(board, &move_gen, depth);
                }

                "uci" => {
                    uci::uci(&mut board, &move_gen)?;
                    break 'main;
                }

                "quit" => break 'main,

                "move" => {
                    for potential_move in arguments {
                        if let Ok(r#move) = Move::try_from(potential_move) {
                            board
                                .make_move(r#move)
                                .unwrap_or_else(|_| panic!("Illegal move '{potential_move}'"));
                        } else {
                            println!("Invalid move '{potential_move}'");
                            break;
                        }
                    }
                }

                _ => {
                    println!("Invalid command '{}'", command);
                }
            }
        }
    }
    Ok(())
}
