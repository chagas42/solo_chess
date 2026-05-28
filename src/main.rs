mod kitty;

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
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
    transmit_pieces();

    let mut board = puzzle_knight();
    let mut cursor = 28;
    let mut selected: Option<usize> = None;
    let mut running = true;

    while running {
        let moves = match selected {
            Some(i) => board.moves(i),
            None => Vec::new(),
        };
        board.render(cursor, selected, &moves);

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
                    if board.0[cursor].is_some() {
                        selected = Some(cursor);
                    }
                }
                Some(from) => {
                    if from == cursor {
                        selected = None;
                    } else if board.is_valid_move(from, cursor) {
                        board.0[cursor] = board.0[from];
                        board.0[from] = None;
                        selected = None;
                    } else if board.0[cursor].is_some() {
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

fn puzzle_knight() -> Board {
    let mut board = Board([None; 64]);
    board.0[28] = Some(Piece::Knight);
    board.0[43] = Some(Piece::Pawn);
    board.0[45] = Some(Piece::Pawn);
    board
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum Piece {
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

pub struct Board([Option<Piece>; 64]);

impl Board {
    fn render(&self, cursor: usize, selected: Option<usize>, moves: &[usize]) {
        let mut buf: Vec<u8> = Vec::with_capacity(16384);

        kitty::clear_placements(&mut buf);
        queue!(buf, cursor::MoveTo(0, 0)).unwrap();
        write!(buf, "Solo Chess\r\n\r\n").unwrap();

        for row in 0..8usize {
            for sub in 0..5usize {
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
                                if self.0[idx].is_some() {
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
                                if self.0[idx].is_some() {
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
                write!(buf, "\r\n").unwrap();
            }
        }

        write!(buf, "\r\n").unwrap();
        write!(
            buf,
            "\u{2191}\u{2193}\u{2190}\u{2192} move   Enter select/move   Esc cancel   q quit\r\n"
        )
        .unwrap();

        for idx in 0..64 {
            if let Some(p) = self.0[idx] {
                let row = idx / 8;
                let col = idx % 8;
                let term_col = (col * CELL_W + 2) as u16;
                let term_row = (BOARD_TOP + row * CELL_H + 1) as u16;
                queue!(buf, cursor::MoveTo(term_col, term_row)).unwrap();
                kitty::place_image(&mut buf, piece_image_id(p), IMG_COLS, IMG_ROWS);
            }
        }

        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        lock.write_all(&buf).unwrap();
        lock.flush().unwrap();
    }

    fn moves(&self, index: usize) -> Vec<usize> {
        let row = index / 8;
        let col = index % 8;
        let piece = match self.0[index] {
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
                        if self.0[r as usize * 8 + c as usize].is_some() {
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
                        if self.0[r as usize * 8 + c as usize].is_some() {
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
                        if self.0[r as usize * 8 + c as usize].is_some() {
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
        let source = match self.0[from] {
            Some(_) => true,
            None => return false,
        };
        let dest = match self.0[to] {
            Some(_) => true,
            None => return false,
        };
        if !source || !dest {
            return false;
        }
        for target in self.moves(from) {
            if target == to {
                return true;
            }
        }
        false
    }
}
