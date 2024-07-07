extern crate chress;

use std::{io::stdin, process::Command};

use chress::board::{color::Color, piece::Piece, r#move::Move, square::Square, Board, START_FEN};
use chress_cli::{perft, uci};

fn main() -> std::io::Result<()> {
    let mut board = Board::new();
    board.load_from_fen(START_FEN).unwrap();

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
                "reset" => board = Board::new(),
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

                "add" => {
                    let Some(piece_str) = arguments.first().cloned() else {
                        println!("Missing arguments for 'add'");
                        break;
                    };

                    if piece_str.len() != 1 {
                        println!("Invalid argument for add '{piece_str}'");
                        break;
                    }

                    let Ok(piece) = Piece::try_from(piece_str.chars().next().unwrap()) else {
                        println!("Invalid argument for add '{piece_str}'");
                        break;
                    };

                    let color = if piece_str.to_uppercase() == piece_str {
                        Color::White
                    } else {
                        Color::Black
                    };

                    let Some(square) = arguments.get(1).cloned() else {
                        println!("Missing arguments for 'add'");
                        break;
                    };

                    let Ok(square) = Square::try_from(square) else {
                        println!("Invalid argument for 'add'");
                        break;
                    };

                    board.add_piece(piece, color, square);
                }

                "rm" => {
                    let Some(square) = arguments.first().cloned() else {
                        println!("Missing arguments for 'rm'");
                        break;
                    };

                    let Ok(square) = Square::try_from(square) else {
                        println!("Invalid argument for 'rm'");
                        break;
                    };

                    for bb in board.piece_bitboards.iter_mut() {
                        *bb &= !square.bitboard();
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

                "dbg" => {
                    println!("Flags: {:08b}", board.flags.0);
                }

                _ => {
                    println!("Invalid command '{}'", command);
                }
            }
        }
    }
    Ok(())
}
