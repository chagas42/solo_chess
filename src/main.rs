fn main() {
    let board = Board([const {None}; 64]);


    board.render();
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
        println!("--------------------------------------------------------------------------------------");
        println!("--------------------------------------------------------------------------------------");
        println!("--------------------------------------------------------------------------------------");
        for (i, _square) in self.0.iter().enumerate() {
            if i % 8 == 0 {
                // print!("X");
                println!();
            }

            match _square {
                Some(Piece::King) => print!("[  K  ]"),
                Some(Piece::Queen) => print!("[  Q  ]"),
                Some(Piece::Tower) => print!("[  T  ]"),
                Some(Piece::Bishop) => print!("[  B  ]"),
                Some(Piece::Knight) => print!("[  C  ]"),
                Some(Piece::Pawn) => print!("[  P  ]"),
                None => print!("[  .  ]")
            }
        }
    println!("\n------------");
    }

}
