mod kitty;
mod puzzle;

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal;
use crossterm::{execute, queue};
use std::io::Write;

const PIECE_KING: &[u8] = include_bytes!("../assets/wk.png");
const PIECE_QUEEN: &[u8] = include_bytes!("../assets/wq.png");
const PIECE_ROOK: &[u8] = include_bytes!("../assets/wr.png");
const PIECE_BISHOP: &[u8] = include_bytes!("../assets/wb.png");
const PIECE_KNIGHT: &[u8] = include_bytes!("../assets/wn.png");
const PIECE_PAWN: &[u8] = include_bytes!("../assets/wp.png");

const IMG_COLS: u32 = 6;
const IMG_ROWS: u32 = 3;
const CELL_W: usize = 10;
const CELL_H: usize = 5;
const BOARD_TOP: usize = 2;
const BOARD_W: usize = 80;
const BOARD_H: usize = 2 + 8 * CELL_H + 2;
const TERM_ROWS: u16 = 46;
const TERM_COLS: u16 = 80;

const PALETTE: [Color; 6] = [
    Color::Rgb { r: 240, g: 90, b: 90 },
    Color::Rgb { r: 240, g: 170, b: 60 },
    Color::Rgb { r: 230, g: 220, b: 90 },
    Color::Rgb { r: 110, g: 210, b: 110 },
    Color::Rgb { r: 90, g: 170, b: 230 },
    Color::Rgb { r: 200, g: 110, b: 230 },
];

fn letter_glyph(c: char) -> [u8; 5] {
    match c {
        'S' => [0b11111, 0b10000, 0b11111, 0b00001, 0b11111],
        'O' => [0b11111, 0b10001, 0b10001, 0b10001, 0b11111],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'C' => [0b11111, 0b10000, 0b10000, 0b10000, 0b11111],
        'H' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        'E' => [0b11111, 0b10000, 0b11111, 0b10000, 0b11111],
        _ => [0; 5],
    }
}

fn render_title(buf: &mut Vec<u8>, frame: u64, term_cols: u16, top_row: u16) {
    const TITLE: &str = "SOLO CHESS";
    const LETTER_W: usize = 5;

    let mut layout: Vec<(usize, [u8; 5])> = Vec::new();
    let mut col = 0usize;
    let mut prev_was_letter = false;
    for c in TITLE.chars() {
        if c == ' ' {
            col += 3;
            prev_was_letter = false;
        } else {
            if prev_was_letter {
                col += 1;
            }
            layout.push((col, letter_glyph(c)));
            col += LETTER_W;
            prev_was_letter = true;
        }
    }
    let total_w = col;
    let left_pad = (term_cols as usize).saturating_sub(total_w) / 2;

    for row in 0..5 {
        for &(letter_col, glyph) in &layout {
            queue!(
                buf,
                cursor::MoveTo((left_pad + letter_col) as u16, top_row + row as u16)
            )
            .unwrap();
            for c in 0..LETTER_W {
                let bit = (glyph[row] >> (4 - c)) & 1;
                if bit == 1 {
                    let idx = (letter_col + c + (frame / 3) as usize) % PALETTE.len();
                    queue!(buf, SetForegroundColor(PALETTE[idx])).unwrap();
                    write!(buf, "\u{2588}").unwrap();
                } else {
                    queue!(buf, ResetColor).unwrap();
                    write!(buf, " ").unwrap();
                }
            }
        }
        queue!(buf, ResetColor).unwrap();
    }
}

fn run_splash(stdout: &mut std::io::Stdout) {
    let (term_cols, term_rows) = terminal::size().unwrap_or((TERM_COLS, TERM_ROWS));
    let top_row = (term_rows as usize).saturating_sub(5) / 2;
    let start = std::time::Instant::now();
    let duration = std::time::Duration::from_millis(1800);
    let mut frame: u64 = 0;

    execute!(
        *stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    while start.elapsed() < duration {
        let mut buf: Vec<u8> = Vec::with_capacity(8192);
        render_title(&mut buf, frame, term_cols, top_row as u16);
        stdout.write_all(&buf).unwrap();
        stdout.flush().unwrap();

        if event::poll(std::time::Duration::from_millis(80)).unwrap() {
            let _ = event::read();
            break;
        }
        frame = frame.wrapping_add(1);
    }

    execute!(
        *stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();
}

const LIGHT_SQ: Color = Color::Rgb { r: 238, g: 238, b: 210 };
const DARK_SQ: Color = Color::Rgb { r: 118, g: 150, b: 86 };
const SELECTED_ON_DARK: Color = Color::Rgb { r: 255, g: 215, b: 60 };
const SELECTED_ON_LIGHT: Color = Color::Rgb { r: 170, g: 110, b: 25 };
const CURSOR_ON_DARK: Color = Color::Rgb { r: 240, g: 240, b: 180 };
const CURSOR_ON_LIGHT: Color = Color::Rgb { r: 95, g: 85, b: 35 };
const DOT_FG: Color = Color::Rgb { r: 70, g: 70, b: 70 };

fn main() {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::SetSize(TERM_COLS, TERM_ROWS),
        cursor::Hide
    )
    .unwrap();

    run_splash(&mut stdout);
    transmit_pieces();

    let (term_cols, term_rows) = terminal::size().unwrap_or((TERM_COLS, TERM_ROWS));
    let h_pad: u16 = ((term_cols as usize).saturating_sub(BOARD_W) / 2) as u16;
    let v_pad: u16 = ((term_rows as usize).saturating_sub(BOARD_H) / 2) as u16;

    let mut session = PuzzleSession::new();
    let mut cursor: usize = 28;
    let mut selected: Option<usize> = None;
    let mut running = true;

    while running {
        let moves = match selected {
            Some(i) if session.board.moves_used[i] < 2 => session.board.moves(i),
            _ => Vec::new(),
        };
        session.board.render(
            cursor,
            selected,
            &moves,
            (session.current_level + 1) as u8,
            session.current_puzzle + 1,
            session.levels[session.current_level].len(),
            h_pad,
            v_pad,
        );

        match event::read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if cursor >= 8 {
                    cursor -= 8;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if cursor < 56 {
                    cursor += 8;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if cursor % 8 > 0 {
                    cursor -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if cursor % 8 < 7 {
                    cursor += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => match selected {
                None => {
                    if session.board.pieces[cursor].is_some() {
                        selected = Some(cursor);
                    }
                }
                Some(from) => {
                    if from == cursor {
                        selected = None;
                    } else if session.board.is_valid_move(from, cursor) {
                        session.board.make_move(from, cursor);
                        selected = None;
                    } else if session.board.pieces[cursor].is_some() {
                        selected = Some(cursor);
                    } else {
                        selected = None;
                    }
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                selected = None;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                session.next();
                cursor = 28;
                selected = None;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                session.prev();
                cursor = 28;
                selected = None;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                session.reset();
                cursor = 28;
                selected = None;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            }) if c.is_ascii_digit() => {
                let level = if c == '0' {
                    9
                } else {
                    (c.to_digit(10).unwrap() - 1) as usize
                };
                session.goto_level(level);
                cursor = 28;
                selected = None;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                running = false;
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode().unwrap();
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
}

const PUZZLES_PER_LEVEL: usize = 3;
const LEVELS: u8 = 10;

struct PuzzleSession {
    levels: Vec<Vec<Board>>,
    current_level: usize,
    current_puzzle: usize,
    board: Board,
}

impl PuzzleSession {
    fn new() -> Self {
        let mut rng = puzzle::Rng::from_time();
        let mut levels = Vec::with_capacity(LEVELS as usize);
        for level in 1..=LEVELS {
            let mut puzzles = Vec::with_capacity(PUZZLES_PER_LEVEL);
            for _ in 0..PUZZLES_PER_LEVEL {
                puzzles.push(puzzle::generate(level, &mut rng));
            }
            levels.push(puzzles);
        }
        let board = levels[0][0].clone();
        PuzzleSession {
            levels,
            current_level: 0,
            current_puzzle: 0,
            board,
        }
    }

    fn reset(&mut self) {
        self.board = self.levels[self.current_level][self.current_puzzle].clone();
    }

    fn next(&mut self) {
        self.current_puzzle += 1;
        if self.current_puzzle >= self.levels[self.current_level].len() {
            self.current_puzzle = 0;
            self.current_level = (self.current_level + 1) % self.levels.len();
        }
        self.reset();
    }

    fn prev(&mut self) {
        if self.current_puzzle == 0 {
            self.current_level = if self.current_level == 0 {
                self.levels.len() - 1
            } else {
                self.current_level - 1
            };
            self.current_puzzle = self.levels[self.current_level].len() - 1;
        } else {
            self.current_puzzle -= 1;
        }
        self.reset();
    }

    fn goto_level(&mut self, level: usize) {
        if level < self.levels.len() {
            self.current_level = level;
            self.current_puzzle = 0;
            self.reset();
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Piece {
    King,
    Queen,
    Tower,
    Bishop,
    Knight,
    Pawn,
}

fn piece_image_id(p: Piece) -> u32 {
    match p {
        Piece::King => 1,
        Piece::Queen => 2,
        Piece::Tower => 3,
        Piece::Bishop => 4,
        Piece::Knight => 5,
        Piece::Pawn => 6,
    }
}

fn transmit_pieces() {
    let mut buf: Vec<u8> = Vec::with_capacity(128 * 1024);
    kitty::transmit_image(&mut buf, 1, PIECE_KING);
    kitty::transmit_image(&mut buf, 2, PIECE_QUEEN);
    kitty::transmit_image(&mut buf, 3, PIECE_ROOK);
    kitty::transmit_image(&mut buf, 4, PIECE_BISHOP);
    kitty::transmit_image(&mut buf, 5, PIECE_KNIGHT);
    kitty::transmit_image(&mut buf, 6, PIECE_PAWN);
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    lock.write_all(&buf).unwrap();
    lock.flush().unwrap();
}

#[derive(Clone)]
pub struct Board {
    pub(crate) pieces: [Option<Piece>; 64],
    pub(crate) moves_used: [u8; 64],
}

impl Board {
    pub fn empty() -> Self {
        Board {
            pieces: [None; 64],
            moves_used: [0; 64],
        }
    }

    pub fn make_move(&mut self, from: usize, to: usize) {
        let new_used = self.moves_used[from] + 1;
        self.pieces[to] = self.pieces[from];
        self.pieces[from] = None;
        self.moves_used[to] = new_used;
        self.moves_used[from] = 0;
    }

    pub fn piece_count(&self) -> usize {
        self.pieces.iter().filter(|p| p.is_some()).count()
    }

    pub fn has_king(&self) -> bool {
        self.pieces.iter().any(|p| matches!(p, Some(Piece::King)))
    }

    pub fn is_won(&self) -> bool {
        if self.piece_count() != 1 {
            return false;
        }
        if !self.has_king() {
            return true;
        }
        self.pieces.iter().any(|p| matches!(p, Some(Piece::King)))
    }

    pub fn has_any_valid_move(&self) -> bool {
        for from in 0..64 {
            if self.pieces[from].is_none() || self.moves_used[from] >= 2 {
                continue;
            }
            for to in self.moves(from) {
                if self.is_valid_move(from, to) {
                    return true;
                }
            }
        }
        false
    }
}


impl Board {
    fn render(
        &self,
        cursor: usize,
        selected: Option<usize>,
        moves: &[usize],
        level: u8,
        puzzle_num: usize,
        total_puzzles: usize,
        h_pad: u16,
        v_pad: u16,
    ) {
        let mut buf: Vec<u8> = Vec::with_capacity(16384);
        let won = self.is_won();
        let stuck = !won && !self.has_any_valid_move();

        kitty::clear_placements(&mut buf);

        let info = format!("Level {}   Puzzle {}/{}", level, puzzle_num, total_puzzles);
        let info_centered = format!("{:^width$}", info, width = BOARD_W);
        queue!(buf, ResetColor, cursor::MoveTo(h_pad, v_pad)).unwrap();
        write!(buf, "{}", info_centered).unwrap();
        queue!(buf, cursor::MoveTo(h_pad, v_pad + 1)).unwrap();
        for _ in 0..BOARD_W {
            write!(buf, " ").unwrap();
        }

        for row in 0..8usize {
            for sub in 0..5usize {
                let term_row = v_pad + (BOARD_TOP + row * CELL_H + sub) as u16;
                queue!(buf, cursor::MoveTo(h_pad, term_row)).unwrap();
                for col in 0..8usize {
                    let idx = row * 8 + col;
                    let light = (row + col) % 2 == 0;
                    let is_cursor = idx == cursor;
                    let is_selected = selected == Some(idx);
                    let is_target = moves.contains(&idx);

                    let bg = if light { LIGHT_SQ } else { DARK_SQ };
                    let label_fg = if light { DARK_SQ } else { LIGHT_SQ };

                    queue!(buf, SetBackgroundColor(bg)).unwrap();

                    let selected_line = if light { SELECTED_ON_LIGHT } else { SELECTED_ON_DARK };
                    let cursor_line = if light { CURSOR_ON_LIGHT } else { CURSOR_ON_DARK };

                    let outline: Option<(Color, [&str; 6])> = if is_selected {
                        Some((selected_line, ["╔", "═", "╗", "║", "╚", "╝"]))
                    } else if is_cursor {
                        Some((cursor_line, ["┌", "─", "┐", "│", "└", "┘"]))
                    } else {
                        None
                    };

                    if let Some((c, chars)) = outline {
                        queue!(buf, SetForegroundColor(c)).unwrap();
                        match sub {
                            0 => {
                                write!(buf, "{}", chars[0]).unwrap();
                                for _ in 0..8 {
                                    write!(buf, "{}", chars[1]).unwrap();
                                }
                                write!(buf, "{}", chars[2]).unwrap();
                            }
                            4 => {
                                write!(buf, "{}", chars[4]).unwrap();
                                for _ in 0..8 {
                                    write!(buf, "{}", chars[1]).unwrap();
                                }
                                write!(buf, "{}", chars[5]).unwrap();
                            }
                            _ => {
                                write!(buf, "{}", chars[3]).unwrap();
                                let art_idx = sub - 1;
                                if self.pieces[idx].is_some() {
                                    write!(buf, "        ").unwrap();
                                } else if is_target && art_idx == 1 {
                                    queue!(buf, SetForegroundColor(DOT_FG)).unwrap();
                                    write!(buf, "   \u{2022}    ").unwrap();
                                } else {
                                    write!(buf, "        ").unwrap();
                                }
                                queue!(buf, SetForegroundColor(c)).unwrap();
                                write!(buf, "{}", chars[3]).unwrap();
                            }
                        }
                    } else {
                        match sub {
                            0 => {
                                if col == 0 {
                                    queue!(buf, SetForegroundColor(label_fg)).unwrap();
                                    write!(buf, " {}        ", 8 - row).unwrap();
                                } else {
                                    write!(buf, "          ").unwrap();
                                }
                            }
                            4 => {
                                if row == 7 {
                                    queue!(buf, SetForegroundColor(label_fg)).unwrap();
                                    let file = (b'a' + col as u8) as char;
                                    write!(buf, "    {}     ", file).unwrap();
                                } else {
                                    write!(buf, "          ").unwrap();
                                }
                            }
                            _ => {
                                let art_idx = sub - 1;
                                if self.pieces[idx].is_some() {
                                    write!(buf, "          ").unwrap();
                                } else if is_target && art_idx == 1 {
                                    queue!(buf, SetForegroundColor(DOT_FG)).unwrap();
                                    write!(buf, "    \u{2022}     ").unwrap();
                                } else {
                                    write!(buf, "          ").unwrap();
                                }
                            }
                        }
                    }
                }
                queue!(buf, ResetColor).unwrap();
            }
        }

        let status_row = v_pad + (BOARD_TOP + 8 * CELL_H) as u16;
        queue!(buf, ResetColor, cursor::MoveTo(h_pad, status_row)).unwrap();
        let status_msg = if won {
            "SOLVED! press n for next puzzle"
        } else if stuck {
            "Stuck \u{2014} press r to reset"
        } else {
            ""
        };
        let status_centered = format!("{:^width$}", status_msg, width = BOARD_W);
        write!(buf, "{}", status_centered).unwrap();

        queue!(buf, cursor::MoveTo(h_pad, status_row + 1)).unwrap();
        let keys = "\u{2191}\u{2193}\u{2190}\u{2192} move | Enter select | n next | p prev | r reset | 1-0 level | q quit";
        let keys_centered = format!("{:^width$}", keys, width = BOARD_W);
        write!(buf, "{}", keys_centered).unwrap();

        for idx in 0..64 {
            if let Some(p) = self.pieces[idx] {
                let row = idx / 8;
                let col = idx % 8;
                let term_col = h_pad + (col * CELL_W + 2) as u16;
                let term_row = v_pad + (BOARD_TOP + row * CELL_H + 1) as u16;
                queue!(buf, cursor::MoveTo(term_col, term_row)).unwrap();
                kitty::place_image(&mut buf, piece_image_id(p), IMG_COLS, IMG_ROWS);
            }
        }

        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        lock.write_all(&buf).unwrap();
        lock.flush().unwrap();
    }

    pub(crate) fn moves(&self, index: usize) -> Vec<usize> {
        let row = index / 8;
        let col = index % 8;
        let piece = match self.pieces[index] {
            Some(p) => p,
            None => return Vec::new(),
        };
        let mut targets = Vec::new();
        match piece {
            Piece::Knight => {
                let offsets = [
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                ];
                for (dr, dc) in offsets {
                    let r = row as i8 + dr;
                    let c = col as i8 + dc;
                    if r >= 0 && r < 8 && c >= 0 && c < 8 {
                        targets.push(r as usize * 8 + c as usize);
                    }
                }
            }
            Piece::King => {
                for dr in -1i8..=1 {
                    for dc in -1i8..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let r = row as i8 + dr;
                        let c = col as i8 + dc;
                        if r >= 0 && r < 8 && c >= 0 && c < 8 {
                            targets.push(r as usize * 8 + c as usize);
                        }
                    }
                }
            }
            Piece::Tower => {
                let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                for (dr, dc) in dirs {
                    let mut r = row as i8 + dr;
                    let mut c = col as i8 + dc;
                    while r >= 0 && r < 8 && c >= 0 && c < 8 {
                        targets.push(r as usize * 8 + c as usize);
                        if self.pieces[r as usize * 8 + c as usize].is_some() {
                            break;
                        }
                        r += dr;
                        c += dc;
                    }
                }
            }
            Piece::Bishop => {
                let dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
                for (dr, dc) in dirs {
                    let mut r = row as i8 + dr;
                    let mut c = col as i8 + dc;
                    while r >= 0 && r < 8 && c >= 0 && c < 8 {
                        targets.push(r as usize * 8 + c as usize);
                        if self.pieces[r as usize * 8 + c as usize].is_some() {
                            break;
                        }
                        r += dr;
                        c += dc;
                    }
                }
            }
            Piece::Queen => {
                let dirs = [
                    (0, 1),
                    (0, -1),
                    (1, 0),
                    (-1, 0),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ];
                for (dr, dc) in dirs {
                    let mut r = row as i8 + dr;
                    let mut c = col as i8 + dc;
                    while r >= 0 && r < 8 && c >= 0 && c < 8 {
                        targets.push(r as usize * 8 + c as usize);
                        if self.pieces[r as usize * 8 + c as usize].is_some() {
                            break;
                        }
                        r += dr;
                        c += dc;
                    }
                }
            }
            Piece::Pawn => {
                for dc in [-1, 1] {
                    let r = row as i8 - 1;
                    let c = col as i8 + dc;
                    if r >= 0 && r < 8 && c >= 0 && c < 8 {
                        targets.push(r as usize * 8 + c as usize);
                    }
                }
            }
        }
        targets
    }

    fn is_valid_move(&self, from: usize, to: usize) -> bool {
        if self.pieces[from].is_none() {
            return false;
        }
        if self.moves_used[from] >= 2 {
            return false;
        }
        match self.pieces[to] {
            Some(Piece::King) => return false,
            Some(_) => {}
            None => return false,
        }
        for target in self.moves(from) {
            if target == to {
                return true;
            }
        }
        false
    }
}
