mod kitty;
mod puzzle;

use crossterm::cursor;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal;
use crossterm::{execute, queue};
use std::io::Write;
use std::time::{Duration, Instant};

const PIECE_KING: &[u8] = include_bytes!("../assets/wk.png");
const PIECE_QUEEN: &[u8] = include_bytes!("../assets/wq.png");
const PIECE_ROOK: &[u8] = include_bytes!("../assets/wr.png");
const PIECE_BISHOP: &[u8] = include_bytes!("../assets/wb.png");
const PIECE_KNIGHT: &[u8] = include_bytes!("../assets/wn.png");
const PIECE_PAWN: &[u8] = include_bytes!("../assets/wp.png");
const CAPTURE_AUDIO: &[u8] = include_bytes!("../assets/capture.mp3");

fn play_capture_sound() {
    std::thread::spawn(|| {
        let players: &[(&str, &[&str])] = &[
            ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet", "-"]),
            ("mpv", &["--no-video", "--really-quiet", "-"]),
            ("mpg123", &["-q", "-"]),
            ("play", &["-q", "-t", "mp3", "-"]),
        ];
        for (cmd, args) in players {
            if let Ok(mut child) = std::process::Command::new(cmd)
                .args(*args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(CAPTURE_AUDIO);
                }
                let _ = child.wait();
                return;
            }
        }
    });
}

const IMG_COLS: u32 = 6;
const IMG_ROWS: u32 = 3;
const CELL_W: usize = 10;
const CELL_H: usize = 5;
const BOARD_TOP: usize = 2;
const BOARD_W: usize = 80;
const BOARD_H: usize = 2 + 8 * CELL_H + 2;
const TERM_ROWS: u16 = 46;
const TERM_COLS: u16 = 80;

const TILE_WHITE: Color = Color::Rgb { r: 245, g: 245, b: 245 };
const TILE_GREEN: Color = Color::Rgb { r: 118, g: 150, b: 86 };

const PAWN_GLYPH: [u16; 12] = [
    0b0000111100000000,
    0b0001111110000000,
    0b0001111110000000,
    0b0000111100000000,
    0b0001111110000000,
    0b0011111111000000,
    0b0011111111000000,
    0b0001111110000000,
    0b0011111111000000,
    0b0111111111100000,
    0b1111111111110000,
    0b1111111111110000,
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

fn render_pawn_mosaic(buf: &mut Vec<u8>, term_cols: u16, top_row: u16) {
    const PIECE_W: usize = 12;
    const PIECE_H: usize = 12;
    const PIXEL_W: usize = 2;

    let total_w = PIECE_W * PIXEL_W;
    let left_pad = (term_cols as usize).saturating_sub(total_w) / 2;

    queue!(buf, SetForegroundColor(TILE_WHITE)).unwrap();
    for row in 0..PIECE_H {
        queue!(
            buf,
            cursor::MoveTo(left_pad as u16, top_row + row as u16)
        )
        .unwrap();
        for col in 0..PIECE_W {
            let bit = (PAWN_GLYPH[row] >> (15 - col)) & 1;
            if bit == 1 {
                write!(buf, "\u{2588}\u{2588}").unwrap();
            } else {
                write!(buf, "  ").unwrap();
            }
        }
    }
    queue!(buf, ResetColor).unwrap();
}

fn render_solo_chess_text(buf: &mut Vec<u8>, term_cols: u16, top_row: u16) {
    const LETTER_W: usize = 5;
    let solo_w = 4 * LETTER_W + 3;
    let chess_w = 5 * LETTER_W + 4;
    let total_w = solo_w + 3 + chess_w;
    let left_pad = (term_cols as usize).saturating_sub(total_w) / 2;

    for row in 0..5 {
        queue!(buf, cursor::MoveTo(left_pad as u16, top_row + row as u16)).unwrap();

        queue!(buf, SetForegroundColor(TILE_WHITE)).unwrap();
        for (i, ch) in "SOLO".chars().enumerate() {
            if i > 0 {
                write!(buf, " ").unwrap();
            }
            let glyph = letter_glyph(ch);
            for c in 0..LETTER_W {
                let bit = (glyph[row] >> (4 - c)) & 1;
                if bit == 1 {
                    write!(buf, "\u{2588}").unwrap();
                } else {
                    write!(buf, " ").unwrap();
                }
            }
        }

        write!(buf, "   ").unwrap();

        queue!(buf, SetForegroundColor(TILE_GREEN)).unwrap();
        for (i, ch) in "CHESS".chars().enumerate() {
            if i > 0 {
                write!(buf, " ").unwrap();
            }
            let glyph = letter_glyph(ch);
            for c in 0..LETTER_W {
                let bit = (glyph[row] >> (4 - c)) & 1;
                if bit == 1 {
                    write!(buf, "\u{2588}").unwrap();
                } else {
                    write!(buf, " ").unwrap();
                }
            }
        }
    }
    queue!(buf, ResetColor).unwrap();
}

fn wait_for_size(stdout: &mut std::io::Stdout) -> bool {
    loop {
        let (cols, rows) = terminal::size().unwrap_or((0, 0));
        if cols >= TERM_COLS && rows >= TERM_ROWS {
            return true;
        }

        execute!(
            *stdout,
            terminal::Clear(terminal::ClearType::All),
            ResetColor,
            cursor::MoveTo(0, 0)
        )
        .unwrap();

        let mut buf = Vec::new();
        write!(buf, "terminal pequeno demais\r\n").unwrap();
        write!(
            buf,
            "precisa de pelo menos {} colunas x {} linhas\r\n",
            TERM_COLS, TERM_ROWS
        )
        .unwrap();
        write!(buf, "atual: {} x {}\r\n", cols, rows).unwrap();
        write!(buf, "\r\n").unwrap();
        write!(buf, "redimensiona a janela ou pressiona q pra sair\r\n").unwrap();
        stdout.write_all(&buf).unwrap();
        stdout.flush().unwrap();

        loop {
            match event::read().unwrap() {
                Event::Resize(_, _) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    ..
                }) => return false,
                _ => continue,
            }
        }
    }
}

fn run_splash(stdout: &mut std::io::Stdout) {
    let (term_cols, term_rows) = terminal::size().unwrap_or((TERM_COLS, TERM_ROWS));
    const PAWN_H: usize = 12;
    const GAP: usize = 2;
    const TEXT_H: usize = 5;
    let total_h = PAWN_H + GAP + TEXT_H;
    let pawn_top = (term_rows as usize).saturating_sub(total_h) / 2;
    let text_top = pawn_top + PAWN_H + GAP;

    execute!(
        *stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    let mut buf: Vec<u8> = Vec::with_capacity(16384);
    render_pawn_mosaic(&mut buf, term_cols, pawn_top as u16);
    render_solo_chess_text(&mut buf, term_cols, text_top as u16);
    stdout.write_all(&buf).unwrap();
    stdout.flush().unwrap();

    let start = std::time::Instant::now();
    let duration = std::time::Duration::from_millis(1800);
    while start.elapsed() < duration {
        if event::poll(std::time::Duration::from_millis(50)).unwrap() {
            let _ = event::read();
            break;
        }
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
const SELECTED_BG: Color = Color::Rgb { r: 246, g: 246, b: 105 };
const CURSOR_BG_LIGHT: Color = Color::Rgb { r: 225, g: 225, b: 175 };
const CURSOR_BG_DARK: Color = Color::Rgb { r: 142, g: 172, b: 105 };
const DOT_FG: Color = Color::Rgb { r: 70, g: 70, b: 70 };
const PIECE_FG: Color = Color::Rgb { r: 248, g: 248, b: 248 };

/// Decide se desenhamos as peças com o protocolo de imagem do Kitty
/// (kitty, Ghostty, WezTerm) ou se caímos no fallback de glifos Unicode.
/// `--ascii`/`--unicode` força o fallback; `--kitty` força as imagens.
fn supports_kitty() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--ascii" || a == "--unicode") {
        return false;
    }
    if args.iter().any(|a| a == "--kitty") {
        return true;
    }
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") || term.contains("ghostty") {
            return true;
        }
    }
    if std::env::var("KITTY_WINDOW_ID").is_ok() || std::env::var("WEZTERM_EXECUTABLE").is_ok() {
        return true;
    }
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        let prog = prog.to_lowercase();
        if prog.contains("ghostty") || prog.contains("wezterm") {
            return true;
        }
    }
    false
}

fn piece_glyph(p: Piece) -> char {
    match p {
        Piece::King => '\u{265A}',
        Piece::Queen => '\u{265B}',
        Piece::Tower => '\u{265C}',
        Piece::Bishop => '\u{265D}',
        Piece::Knight => '\u{265E}',
        Piece::Pawn => '\u{265F}',
    }
}

fn first_piece(board: &Board) -> Option<usize> {
    (0..64).find(|&i| board.pieces[i].is_some())
}

/// Casa ocupada mais próxima a partir de `cur` na direção `(dr, dc)`.
/// Considera todas as peças no semiplano daquela direção e escolhe a
/// de menor distância no eixo (desempate pela distância perpendicular),
/// de modo que setas repetidas alcançam qualquer peça do tabuleiro.
fn nearest_in_direction(board: &Board, cur: usize, dr: i32, dc: i32) -> Option<usize> {
    let cr = (cur / 8) as i32;
    let cc = (cur % 8) as i32;
    let mut best: Option<(i32, i32, usize)> = None;
    for idx in 0..64 {
        if idx == cur || board.pieces[idx].is_none() {
            continue;
        }
        let r = (idx / 8) as i32;
        let c = (idx % 8) as i32;
        let along = (r - cr) * dr + (c - cc) * dc;
        if along <= 0 {
            continue;
        }
        let perp = if dr != 0 { (c - cc).abs() } else { (r - cr).abs() };
        match best {
            Some((a, p, _)) if (along, perp) >= (a, p) => {}
            _ => best = Some((along, perp, idx)),
        }
    }
    best.map(|(_, _, idx)| idx)
}

#[derive(PartialEq, Clone, Copy)]
enum Phase {
    Inspect,
    Solve,
    Done,
    Stuck,
}

#[derive(PartialEq, Clone, Copy)]
enum Hints {
    Full,
    Off,
}

/// Estado de uma tentativa do puzzle atual: fase do treino + cronômetros.
struct Attempt {
    phase: Phase,
    inspect_start: Instant,
    solve_start: Instant,
    inspect_secs: u64,
    solve_secs: u64,
    moves: u32,
    clean: bool,
}

impl Attempt {
    fn new() -> Self {
        let now = Instant::now();
        Attempt {
            phase: Phase::Inspect,
            inspect_start: now,
            solve_start: now,
            inspect_secs: 0,
            solve_secs: 0,
            moves: 0,
            clean: true,
        }
    }
}

fn fmt_clock(secs: u64) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn write_info_line(buf: &mut Vec<u8>, info: &str, h_pad: u16, v_pad: u16) {
    let centered = format!("{:^width$}", info, width = BOARD_W);
    queue!(buf, ResetColor, cursor::MoveTo(h_pad, v_pad)).unwrap();
    write!(buf, "{}", centered).unwrap();
}

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

    if !wait_for_size(&mut stdout) {
        execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        terminal::disable_raw_mode().unwrap();
        return;
    }

    let kitty = supports_kitty();

    run_splash(&mut stdout);
    if kitty {
        transmit_pieces();
    }

    let (term_cols, term_rows) = terminal::size().unwrap_or((TERM_COLS, TERM_ROWS));
    let mut h_pad: u16 = ((term_cols as usize).saturating_sub(BOARD_W) / 2) as u16;
    let mut v_pad: u16 = ((term_rows as usize).saturating_sub(BOARD_H) / 2) as u16;

    let mut session = PuzzleSession::new();
    let mut cursor: usize = first_piece(&session.board).unwrap_or(28);
    let mut selected: Option<usize> = None;
    let mut attempt = Attempt::new();
    let mut streak: u32 = 0;
    let mut hints = Hints::Full;
    let mut running = true;
    let mut dirty = true;
    let mut last_secs = u64::MAX;

    while running {
        let level = (session.current_level + 1) as u8;
        let puzzle_num = session.current_puzzle + 1;
        let total = session.levels[session.current_level].len();

        let cur_secs = match attempt.phase {
            Phase::Inspect => attempt.inspect_start.elapsed().as_secs(),
            Phase::Solve => attempt.solve_start.elapsed().as_secs(),
            Phase::Done => attempt.solve_secs,
            Phase::Stuck => 0,
        };

        let clock = match attempt.phase {
            Phase::Inspect => format!("inspecao {}", fmt_clock(cur_secs)),
            Phase::Solve => format!("resolvendo {}", fmt_clock(cur_secs)),
            Phase::Done => format!("resolvido {}", fmt_clock(attempt.solve_secs)),
            Phase::Stuck => "sem saida".to_string(),
        };
        let info_line = format!("Nivel {}  Puzzle {}/{}   {}", level, puzzle_num, total, clock);

        let status_line = match attempt.phase {
            Phase::Inspect => {
                "Inspecione e planeje a sequencia \u{2014} Enter para comecar".to_string()
            }
            Phase::Solve => String::new(),
            Phase::Stuck => "Sem saida \u{2014} r para tentar de novo".to_string(),
            Phase::Done => format!(
                "Resolvido! inspecao {}s \u{00b7} execucao {}s \u{00b7} {} lances \u{00b7} streak {} \u{2014} n proximo",
                attempt.inspect_secs, attempt.solve_secs, attempt.moves, streak
            ),
        };

        let show_hints = attempt.phase == Phase::Solve && hints == Hints::Full;
        let moves = match selected {
            Some(i) if show_hints && session.board.moves_used[i] < 2 => session.board.moves(i),
            _ => Vec::new(),
        };

        if dirty {
            session
                .board
                .render(cursor, selected, &moves, &info_line, &status_line, kitty, h_pad, v_pad);
            dirty = false;
            last_secs = cur_secs;
        } else if matches!(attempt.phase, Phase::Inspect | Phase::Solve) && cur_secs != last_secs {
            last_secs = cur_secs;
            let mut buf: Vec<u8> = Vec::with_capacity(256);
            write_info_line(&mut buf, &info_line, h_pad, v_pad);
            let mut lock = stdout.lock();
            lock.write_all(&buf).unwrap();
            lock.flush().unwrap();
        }

        if !event::poll(Duration::from_millis(200)).unwrap() {
            continue;
        }

        match event::read().unwrap() {
            Event::Key(KeyEvent { code, kind: KeyEventKind::Press, .. }) => {
                dirty = true;
                match code {
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        let (dr, dc) = match code {
                            KeyCode::Up => (-1, 0),
                            KeyCode::Down => (1, 0),
                            KeyCode::Left => (0, -1),
                            _ => (0, 1),
                        };
                        if let Some(next) = nearest_in_direction(&session.board, cursor, dr, dc) {
                            cursor = next;
                        }
                    }
                    KeyCode::Enter => match attempt.phase {
                        Phase::Inspect => {
                            attempt.inspect_secs = attempt.inspect_start.elapsed().as_secs();
                            attempt.solve_start = Instant::now();
                            attempt.phase = Phase::Solve;
                        }
                        Phase::Solve => match selected {
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
                                    play_capture_sound();
                                    attempt.moves += 1;
                                    selected = None;
                                    if session.board.is_won() {
                                        attempt.solve_secs =
                                            attempt.solve_start.elapsed().as_secs();
                                        attempt.phase = Phase::Done;
                                        if attempt.clean {
                                            streak += 1;
                                        }
                                    } else if !session.board.has_any_valid_move() {
                                        attempt.clean = false;
                                        attempt.phase = Phase::Stuck;
                                    }
                                } else if session.board.pieces[cursor].is_some() {
                                    selected = Some(cursor);
                                } else {
                                    selected = None;
                                }
                            }
                        },
                        _ => {}
                    },
                    KeyCode::Esc => {
                        selected = None;
                    }
                    KeyCode::Char('h') => {
                        hints = if hints == Hints::Full { Hints::Off } else { Hints::Full };
                    }
                    KeyCode::Char('n') => {
                        session.next();
                        cursor = first_piece(&session.board).unwrap_or(28);
                        selected = None;
                        attempt = Attempt::new();
                    }
                    KeyCode::Char('p') => {
                        session.prev();
                        cursor = first_piece(&session.board).unwrap_or(28);
                        selected = None;
                        attempt = Attempt::new();
                    }
                    KeyCode::Char('r') => {
                        session.reset();
                        cursor = first_piece(&session.board).unwrap_or(28);
                        selected = None;
                        attempt = Attempt::new();
                        streak = 0;
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let lvl = if c == '0' { 9 } else { (c.to_digit(10).unwrap() - 1) as usize };
                        session.goto_level(lvl);
                        cursor = first_piece(&session.board).unwrap_or(28);
                        selected = None;
                        attempt = Attempt::new();
                        streak = 0;
                    }
                    KeyCode::Char('q') => {
                        running = false;
                    }
                    _ => {
                        dirty = false;
                    }
                }
            }
            Event::Resize(new_cols, new_rows) => {
                if new_cols < TERM_COLS || new_rows < TERM_ROWS {
                    if !wait_for_size(&mut stdout) {
                        running = false;
                        continue;
                    }
                    if kitty {
                        transmit_pieces();
                    }
                }
                let (cols, rows) = terminal::size().unwrap_or((TERM_COLS, TERM_ROWS));
                h_pad = ((cols as usize).saturating_sub(BOARD_W) / 2) as u16;
                v_pad = ((rows as usize).saturating_sub(BOARD_H) / 2) as u16;
                dirty = true;
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
        info_line: &str,
        status_line: &str,
        kitty: bool,
        h_pad: u16,
        v_pad: u16,
    ) {
        let mut buf: Vec<u8> = Vec::with_capacity(16384);

        if kitty {
            kitty::clear_placements(&mut buf);
        }

        write_info_line(&mut buf, info_line, h_pad, v_pad);
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

                    let base_bg = if light { LIGHT_SQ } else { DARK_SQ };
                    let bg = if is_selected {
                        SELECTED_BG
                    } else if is_cursor {
                        if light { CURSOR_BG_LIGHT } else { CURSOR_BG_DARK }
                    } else {
                        base_bg
                    };
                    let label_fg = if light { DARK_SQ } else { LIGHT_SQ };

                    queue!(buf, SetBackgroundColor(bg)).unwrap();

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
                            if self.pieces[idx].is_some() {
                                write!(buf, "          ").unwrap();
                            } else if is_target && sub == 2 {
                                queue!(buf, SetForegroundColor(DOT_FG)).unwrap();
                                write!(buf, "    \u{2022}     ").unwrap();
                            } else {
                                write!(buf, "          ").unwrap();
                            }
                        }
                    }
                }
                queue!(buf, ResetColor).unwrap();
            }
        }

        let status_row = v_pad + (BOARD_TOP + 8 * CELL_H) as u16;
        queue!(buf, ResetColor, cursor::MoveTo(h_pad, status_row)).unwrap();
        let status_centered = format!("{:^width$}", status_line, width = BOARD_W);
        write!(buf, "{}", status_centered).unwrap();

        queue!(buf, cursor::MoveTo(h_pad, status_row + 1)).unwrap();
        let keys = "setas mover | Enter selec | h dicas | n prox | p ant | r reset | 1-0 nivel | q sair";
        let keys_centered = format!("{:^width$}", keys, width = BOARD_W);
        write!(buf, "{}", keys_centered).unwrap();

        for idx in 0..64 {
            if let Some(p) = self.pieces[idx] {
                let row = idx / 8;
                let col = idx % 8;
                if kitty {
                    let term_col = h_pad + (col * CELL_W + 2) as u16;
                    let term_row = v_pad + (BOARD_TOP + row * CELL_H + 1) as u16;
                    queue!(buf, cursor::MoveTo(term_col, term_row)).unwrap();
                    kitty::place_image(&mut buf, piece_image_id(p), IMG_COLS, IMG_ROWS);
                } else {
                    let light = (row + col) % 2 == 0;
                    let is_selected = selected == Some(idx);
                    let is_cursor = idx == cursor;
                    let bg = if is_selected {
                        SELECTED_BG
                    } else if is_cursor {
                        if light { CURSOR_BG_LIGHT } else { CURSOR_BG_DARK }
                    } else if light {
                        LIGHT_SQ
                    } else {
                        DARK_SQ
                    };
                    let term_col = h_pad + (col * CELL_W + 4) as u16;
                    let term_row = v_pad + (BOARD_TOP + row * CELL_H + 2) as u16;
                    queue!(
                        buf,
                        cursor::MoveTo(term_col, term_row),
                        SetBackgroundColor(bg),
                        SetForegroundColor(PIECE_FG)
                    )
                    .unwrap();
                    write!(buf, "{}", piece_glyph(p)).unwrap();
                    queue!(buf, ResetColor).unwrap();
                }
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
