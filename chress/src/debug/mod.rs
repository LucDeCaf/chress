use crate::{
    board::{r#move::Move, Board},
    move_gen::MoveGen,
};

pub fn perft(board: Board, move_gen: &MoveGen, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    move_gen.legal_moves(&board, &mut moves);

    let mut count = 0;

    for mv in moves {
        let mut b = board;
        b.make_move(mv).unwrap();

        count += perft(b, move_gen, depth - 1);
    }

    count
}

pub fn divide(mut board: Board, move_gen: &MoveGen, depth: usize) -> (u64, Vec<(Move, u64)>) {
    let mut total = 0;
    let mut results = Vec::new();

    let mut moves = Vec::new();
    move_gen.legal_moves(&board, &mut moves);

    for mv in moves {
        let md = board.make_move(mv).unwrap();
        let count = perft(board, move_gen, depth - 1);
        board.unmake_move(md).unwrap();

        total += count;
        results.push((mv, count));
    }

    (total, results)
}
