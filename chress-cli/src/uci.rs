use std::{error::Error, fmt::Display, io::stdin};

use chress::{
    board::{Board, START_FEN},
    r#move::Move,
};

#[derive(Debug, PartialEq)]
enum Command {
    Uci,
    UciNewGame,
    IsReady,
    Position(Vec<String>),
    Go,
    Stop,
    Quit,
    SetOption(String, Option<String>),
}

#[derive(Debug)]
struct ParseCommandError;

impl Display for ParseCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad command string")
    }
}

impl Error for ParseCommandError {}

impl TryFrom<&str> for Command {
    type Error = ParseCommandError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut parts = value.split_whitespace();
        let Some(cmd) = parts.next() else {
            return Err(ParseCommandError);
        };

        match cmd {
            "uci" => Ok(Command::Uci),
            "setoption" => {
                let _ = parts.next();
                let Some(name) = parts.next() else {
                    return Err(ParseCommandError);
                };

                let _ = parts.next();
                let value = parts.next().map(|value| value.to_owned());

                Ok(Command::SetOption(name.to_owned(), value))
            }
            "ucinewgame" => Ok(Command::UciNewGame),
            "isready" => Ok(Command::IsReady),
            "position" => Ok(Command::Position(
                parts.map(|s| s.to_owned()).collect::<Vec<String>>(),
            )),
            "go" => Ok(Command::Go),
            "stop" => Ok(Command::Stop),
            "quit" => Ok(Command::Quit),
            _ => Err(ParseCommandError),
        }
    }
}

fn process_command(command: &Command, board: &mut Board) -> Option<String> {
    match command {
        Command::Uci => Some(
            String::from("id name Chress\n")
                + "id author LucDeCaf\n"
                + "option name Hash type spin default 1 min 1 max 2048\n"
                + "uciok",
        ),
        Command::SetOption(_name, _value) => None,
        Command::IsReady => Some(String::from("readyok")),
        Command::UciNewGame => {
            *board = Board::default();
            Some(String::from("readyok"))
        }
        Command::Position(moves) => {
            for r#move in moves {
                if r#move == "startpos" {
                    board.load_from_fen(START_FEN).unwrap();
                    return None;
                }

                let r#move = Move::try_from(&r#move[..])
                    .expect("UCI move should be in long algebraic notation");

                board.make_move(r#move).expect("UCI move should be legal");
            }

            None
        }
        Command::Go => Some(String::from("bestmove 0000")),
        Command::Stop => None,
        Command::Quit => None,
    }
}

pub fn uci(board: &mut Board) -> std::io::Result<()> {
    let mut input = String::new();

    println!("{}", process_command(&Command::Uci, board).unwrap());

    loop {
        input.clear();

        stdin().read_line(&mut input)?;

        let Ok(command) = Command::try_from(input.trim()) else {
            continue;
        };

        if command == Command::Quit {
            break;
        }

        if let Some(response) = process_command(&command, board) {
            println!("{}", response);
        }
    }

    Ok(())
}
