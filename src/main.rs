mod board;

use board::*;

fn main() {
    println!("Enter a board");
    let mut input_buffer = String::new();
    while !input_buffer.ends_with("\n\n") {
        std::io::stdin()
            .read_line(&mut input_buffer)
            .expect("Input contained illegal characters");
    }
    let board = Board::parse(&input_buffer).expect("Input is not a valid board");
    println!("{}", board.write());
    println!("possible moves");
    for next_board in board.possible_moves(Player::Max) {
        println!("{}", next_board.write());
    }
}
