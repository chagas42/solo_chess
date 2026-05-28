use crate::{Board, Piece};
use std::time::{SystemTime, UNIX_EPOCH};

const NON_KING: [Piece; 5] = [
    Piece::Queen,
    Piece::Tower,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
];

pub struct Rng(u64);

impl Rng {
    pub fn from_time() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Rng(nanos ^ 0x9E3779B97F4A7C15)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    pub fn range(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next() % (n as u64)) as usize
    }

    pub fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.range(items.len())]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.range(i + 1);
            items.swap(i, j);
        }
    }
}

pub fn num_pieces_for_level(level: u8) -> usize {
    (3 + level as usize).min(13)
}

fn find_reverse_origins(board: &Board, piece: Piece, target: usize) -> Vec<usize> {
    let mut origins = Vec::new();
    for origin in 0..64 {
        if origin == target || board.pieces[origin].is_some() {
            continue;
        }
        let mut probe = board.clone();
        probe.pieces[origin] = Some(piece);
        if probe.moves(origin).contains(&target) {
            origins.push(origin);
        }
    }
    origins
}

pub fn generate(level: u8, rng: &mut Rng) -> Board {
    for _ in 0..200 {
        if let Some(b) = try_generate(level, rng) {
            return b;
        }
    }
    let mut fallback = Board::empty();
    fallback.pieces[28] = Some(Piece::King);
    fallback.pieces[36] = Some(Piece::Pawn);
    fallback
}

fn try_generate(level: u8, rng: &mut Rng) -> Option<Board> {
    let num_pieces = num_pieces_for_level(level);
    let mut board = Board::empty();

    let king_pos = rng.range(64);
    board.pieces[king_pos] = Some(Piece::King);
    board.moves_used[king_pos] = 0;

    for _ in 0..(num_pieces - 1) {
        let mut candidates: Vec<usize> = (0..64)
            .filter(|&i| board.pieces[i].is_some() && board.moves_used[i] < 2)
            .collect();
        if candidates.is_empty() {
            return None;
        }
        rng.shuffle(&mut candidates);

        let mut added = false;
        for mover_idx in candidates {
            let mover_piece = board.pieces[mover_idx].unwrap();
            let mover_will_make = board.moves_used[mover_idx];

            let origins = find_reverse_origins(&board, mover_piece, mover_idx);
            if origins.is_empty() {
                continue;
            }

            let origin = origins[rng.range(origins.len())];
            let captured = rng.pick(&NON_KING);

            board.pieces[origin] = Some(mover_piece);
            board.moves_used[origin] = mover_will_make + 1;
            board.pieces[mover_idx] = Some(captured);
            board.moves_used[mover_idx] = 0;

            added = true;
            break;
        }

        if !added {
            return None;
        }
    }

    board.moves_used = [0; 64];
    Some(board)
}
