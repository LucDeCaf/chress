pub fn perft(&mut self, depth: usize) -> u64 {
    let mut results = 0;

    let moves = self.pseudolegal_moves();

    if depth == 0 {
        return 1;
    }

    if depth == 1 {
        return moves
            .into_iter()
            .filter(|mv| self.is_legal_move(*mv))
            .count() as u64;
    }

    for r#move in moves {
        self.make_move(r#move).unwrap();

        let king_square = Square::ALL[self
            .bitboard(Piece::King, self.active_color.inverse())
            .0
            .trailing_zeros() as usize];

        let in_check = self.square_attacked_by(king_square, self.active_color);

        if !in_check {
            results += self.perft(depth - 1);
        }

        self.unmake_move().unwrap();
    }

    results
}

pub fn divide(&mut self, depth: usize) -> (u64, Vec<(Move, u64)>) {
    let mut total = 0;
    let moves = self.pseudolegal_moves();
    let mut results = Vec::with_capacity(moves.len());

    for r#move in moves {
        self.make_move(r#move).unwrap();
        let king_square = Square::ALL[self
            .bitboard(Piece::King, self.active_color.inverse())
            .0
            .trailing_zeros() as usize];

        let in_check = self.square_attacked_by(king_square, self.active_color);

        if !in_check {
            let count = self.perft(depth - 1);
            total += count;
            results.push((r#move, count));
        }

        self.unmake_move().unwrap();
    }

    results.sort_by(|(a, _), (b, _)| a.cmp(b));

    (total, results)
}

pub fn perft_parallel(&mut self, depth: usize) -> u64 {
    let results = Arc::new(Mutex::new(AtomicU64::new(0)));
    let moves = self.pseudolegal_moves();

    if depth == 0 {
        return 1;
    }

    if depth == 1 {
        return moves
            .into_iter()
            .filter(|mv| self.is_legal_move(*mv))
            .count() as u64;
    }

    let mut handles = Vec::new();

    for r#move in moves {
        self.make_move(r#move).unwrap();

        let king_square = Square::ALL[self
            .bitboard(Piece::King, self.active_color.inverse())
            .0
            .trailing_zeros() as usize];

        let in_check = self.square_attacked_by(king_square, self.active_color);

        if !in_check {
            let cloned_board = self.clone();
            let results = Arc::clone(&results);

            handles.push(thread::spawn(move || {
                let mut board = cloned_board;

                let perft = board.perft(depth - 1);

                let results = results.lock().unwrap();
                results.fetch_add(perft, Ordering::Relaxed);
            }));
        }

        self.unmake_move().unwrap();
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let results = results.lock().unwrap();
    results.load(Ordering::Relaxed)
}
