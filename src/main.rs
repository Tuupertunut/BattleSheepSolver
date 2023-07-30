use battle_sheep_solver::{
    board::{Board, Player},
    choose_move,
};
use std::time::{Duration, Instant};

fn main() {
    /* Game mode is given as a command line argument. */
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 || (args[1] != "-p" && args[1] != "-w") {
        panic!(
            "
            Usage: {} {{-p|-w}}
            -p: play against the AI
            -w: watch two AIs play against one another
            ",
            args[0]
        );
    }
    let human_max = match args[1].as_str() {
        "-p" => true,
        "-w" => false,
        _ => unreachable!(),
    };

    println!("Enter a starting board (finish with an empty line)");
    let mut board = read_board_from_user();
    println!("{}", board.write(true));

    /* Min always starts. */
    let mut player = Player::Min;

    let mut turns = 0;
    let mut total_duration = Duration::ZERO;

    /* The game loop. One iteration means one turn. */
    loop {
        let start_time = Instant::now();

        /* The player chooses a move. */
        let (next_board, val, visited) = choose_move(player, &board, 7, i32::MIN + 1, i32::MAX);
        let value = player.sign() * val;

        match next_board {
            None => {
                /* The player could not choose a move, so the game is over. */
                println!();
                if value > 0 {
                    println!("Max won!");
                } else if value < 0 {
                    println!("Min won!")
                } else {
                    println!("Draw!")
                }
                println!(
                    "(average turn took {:?})",
                    total_duration.checked_div(turns).unwrap_or(Duration::ZERO)
                );

                break;
            }
            Some(next_board) => {
                let duration = start_time.elapsed();

                println!();
                println!(
                    "{}'s turn",
                    match player {
                        Player::Min => "Min",
                        Player::Max => "Max",
                    }
                );
                println!(
                    "took {:?}, evaluated {} boards, value {}",
                    duration, visited, value
                );
                println!("{}", next_board.write(true));

                total_duration += duration;
                turns += 1;

                /* Setting up the next turn. */
                if human_max {
                    /* Max is a human player (the user). Their whole turn is played just by asking
                     * them for a board. After that it's Min's turn again. */
                    println!();
                    println!("Max's turn");
                    println!("Enter a board (finish with an empty line)");
                    board = read_board_from_user();
                    println!("{}", board.write(true));

                    player = Player::Min;
                } else {
                    /* The next turn is played by the opposite player. */
                    board = next_board;
                    player = match player {
                        Player::Min => Player::Max,
                        Player::Max => Player::Min,
                    };
                }
            }
        }
    }
}

fn read_board_from_user() -> Board {
    let mut input_buffer = String::new();
    while !input_buffer.ends_with("\n\n") {
        std::io::stdin()
            .read_line(&mut input_buffer)
            .expect("Input contained illegal characters");
    }
    return Board::parse(&input_buffer).expect("Input is not a valid board");
}
