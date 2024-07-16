use std::{
    io::BufReader,
    path::Path,
    process::{ChildStdin, ChildStdout, Command, Stdio},
};

use chress::board::{color::Color, r#move::Move};

pub struct Engine {
    pub id: String,
    pub stdin: ChildStdin,
    pub stdout: BufReader<ChildStdout>,
}

impl Engine {
    pub fn new(id: String, path: &Path) -> std::io::Result<Self> {
        let process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self {
            id,
            stdin: process.stdin.unwrap(),
            stdout: BufReader::new(process.stdout.unwrap()),
        })
    }
}

pub struct GameLog {
    pub result: Color,
    pub moves: Vec<Move>,
}

pub struct Session {
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub engine_1: Engine,
    pub engine_2: Engine,
    pub games: Vec<GameLog>,
}
