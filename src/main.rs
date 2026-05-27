fn main() {
    let board = Board([None; 64]);


    board.render();
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

pub struct Board([Option<Piece>; 64]);

impl Board {
    fn render(&self) {
        println!("-------------solo----------------------------------------");
        println!("------------------------chess----------------------------");
        println!("------------------------------------puzzle---------------");

        let letters = "ABCDEFGH";
        print!("   ");
        for letter in letters.chars() {
            print!("   {}   ", letter);
        }
        for (i, _square) in self.0.iter().enumerate() {

            if i % 8 == 0 {
                println!();
                print!("{}  ", i/8 + 1);
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
