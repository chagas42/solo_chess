fn main() {
    println!("Hello, world!");
}

enum Piece {
    King,
    Queen,
    Tower,
    Bishop,
    Knight,
    Pawn,
}

pub struct Board([Option<Piece>; 64]);

impl Board {
    fn render(&self) {
        for (i, square) in self.0.iter().enumerate() {
            if i % 8 == 0 && i != 0 {
                println!();
            } else {
                println!(". test");
            }
        }
    }
}
