extern crate chress;

use std::{io::stdin, process::Command};

use chress::{
    board::{Board, START_FEN},
    r#move::Move,
};
use chress_cli::{perft, uci};

fn main() -> std::io::Result<()> {
    let mut board = Board::default();

    let mut input = String::new();

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
                "startpos" => board.load_from_fen(START_FEN).unwrap(),
                "load" => {
                    if arguments[0] == "fen" {
                        let arguments = &arguments[1..];

                        if let Err(parse_error) = board.load_from_fen(&arguments.join(" ")) {
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
                    if let Err(unmake_move_error) = board.unmake_move() {
                        println!("Error: {}", unmake_move_error);
                    }
                }

                "moves" => {
                    let mut moves = board.legal_moves();
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

                    perft::perft(&mut board, depth);
                }

                "uci" => {
                    uci::uci(&mut board)?;
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
