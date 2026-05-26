# solo_chess

A terminal port of [chess.com's Solo Chess](https://www.chess.com/solo-chess) puzzle, written in Rust.

Solo Chess is a single-player chess variant: every move must be a capture, each piece can capture at most twice, and if there's a king it must be the last piece standing. It's a focused way to train look-ahead — planning a chain of moves before committing to the first one.

Inspired by playing it on chess.com and wanting a distraction-free, keyboard-only version that runs in the terminal.

## Status

Early development. Not playable yet.

| Module        | Responsibility                                       | Status   |
|---------------|------------------------------------------------------|----------|
| `piece.rs`    | Piece types, movement rules, capture limits          | Planned  |
| `board.rs`    | 8×8 board state, algebraic coordinates               | Planned  |
| `game.rs`     | Solo Chess rules, validation, win/deadlock detection | Planned  |
| `puzzle.rs`   | Hardcoded puzzles, puzzle loading                    | Planned  |
| `renderer.rs` | In-place ASCII/Unicode rendering                     | Planned  |
| `input.rs`    | Command parser (REPL first, then TUI)                | Planned  |

Progress is tracked under [Milestones](../../milestones).

## Rules

1. Every move must be a capture — moving to an empty square is not allowed.
2. Each piece can capture at most twice. After two captures, it is spent and cannot move again.
3. The king cannot be captured. If a king is present, it must be the last piece on the board.
4. Pieces move according to standard chess rules — pawn diagonal-forward capture, knight L-jump, rook/bishop/queen with blocking, king one square.
5. The puzzle is solved when one piece remains. If no captures are possible with more than one piece on the board, the puzzle is in a dead end and must be restarted.

## Roadmap

The project is being built as a walking skeleton — the smallest visible thing first, then layered features. Each milestone below maps to a GitHub Milestone with its own issues.

- **M1 — Walking skeleton**: empty board renders in the terminal
- **M2 — Pieces on the board**: piece types, Unicode glyphs, Board struct
- **M3 — First playable move**: Knight movement + REPL input
- **M4 — All piece movements**: ray casting, knight jumps, pawn captures
- **M5 — Solo Chess rules**: 2-capture limit, king protection, win, deadlock
- **M6 — Puzzles + REPL polish**: puzzle library, undo, in-place redraw
- **M7 — TUI mode**: arrow-key navigation, alternate screen, color
- **M8 — Distribution**: crates.io, prebuilt binaries, demo recording

## Design decisions

- **Walking skeleton over bottom-up.** Ship something visible at every step. Refactoring Rust is cheap; momentum is not.
- **Unicode chess glyphs (♚♛♜♝♞♟) over letter codes.** More readable in modern monospace terminals. Letters reserved as a fallback if glyph alignment fails on a target terminal.
- **REPL input first, TUI second.** Line-based `e4 d6` commands until the rules engine is solid, then arrow-key navigation via `crossterm`. Separating input source from game logic keeps the migration cheap.
- **In-place redraw.** Every state change repaints the same screen region — no scrolling history, no extra boards stacking down the terminal.
- **Hardcoded puzzles first.** Procedural puzzle generation is NP-hard and not on the critical path for the training-tool goal. A small curated library beats a flaky generator.

## Building

```bash
git clone git@github.com:chagas42/solo_chess.git
cd solo_chess
cargo run
```

Requires Rust 1.85+ (edition 2024).

## Contributing

Early days — open an issue before submitting a pull request, the architecture may still change.

## License

MIT
