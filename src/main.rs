mod board;

use std::time::Instant;

use board::*;

fn main() {
    println!("Enter a board");
    let mut input_buffer = String::new();
    while !input_buffer.ends_with("\n\n") {
        std::io::stdin()
            .read_line(&mut input_buffer)
            .expect("Input contained illegal characters");
    }
    let mut board = Board::parse(&input_buffer).expect("Input is not a valid board");
    println!("{}", board.write(true));

    /* Two AIs, Min and Max play the game, alternating turns. Min starts. */
    for &player in [Player::Min, Player::Max].iter().cycle() {
        let start_time = Instant::now();

        /* Here we tell each player how to choose the next turn. This is basic minimax algorithm
         * without optimizations.
         *
         * The min... and max... functions are implemented so that we need to give them a function
         * that evaluates the boards. That's why the function calls are nested. At the depth of 4
         * calls the heuristic evaluation function is used. */
        let (next_board, value, visited) = match player {
            Player::Min => min_choose(&board, |board_1| {
                let (_, value_1, visited_1) = max_choose(board_1, |board_2| {
                    let (_, value_2, visited_2) = min_choose(board_2, |board_3| {
                        let (_, value_3, visited_3) = max_choose(board_3, |board_4| {
                            return (board_4.heuristic_evaluate(), 1);
                        });
                        return (value_3, visited_3);
                    });
                    return (value_2, visited_2);
                });
                return (value_1, visited_1);
            }),
            Player::Max => max_choose(&board, |board_1| {
                let (_, value_1, visited_1) = min_choose(board_1, |board_2| {
                    let (_, value_2, visited_2) = max_choose(board_2, |board_3| {
                        let (_, value_3, visited_3) = min_choose(board_3, |board_4| {
                            return (board_4.heuristic_evaluate(), 1);
                        });
                        return (value_3, visited_3);
                    });
                    return (value_2, visited_2);
                });
                return (value_1, visited_1);
            }),
        };
        match next_board {
            None => {
                if value > 0 {
                    println!("\nMax won!");
                } else if value < 0 {
                    println!("\nMin won!")
                } else {
                    println!("\nDraw!")
                }
                break;
            }
            Some(next_board) => {
                println!(
                    "\n{}'s turn\ntook {:?}, evaluated {} boards, value {}\n{}",
                    match player {
                        Player::Min => "Min",
                        Player::Max => "Max",
                    },
                    start_time.elapsed(),
                    visited,
                    value,
                    next_board.write(true)
                );
                board = next_board;
            }
        }
    }
}

fn min_choose<F>(board: &Board, evaluate_next: F) -> (Option<Board>, i32, u64)
where
    F: Fn(&Board) -> (i32, u64),
{
    let mut chosen_move = None;
    let mut min_value = i32::MAX;
    let mut total_visited = 0;

    for next_board in board.possible_moves(Player::Min) {
        let (value, visited) = evaluate_next(&next_board);
        if value < min_value {
            min_value = value;
            chosen_move = Some(next_board);
        }
        total_visited += visited;
    }

    if chosen_move == None {
        min_value = board.heuristic_evaluate();
        total_visited = 1;
    }

    return (chosen_move, min_value, total_visited);
}

fn max_choose<F>(board: &Board, evaluate_next: F) -> (Option<Board>, i32, u64)
where
    F: Fn(&Board) -> (i32, u64),
{
    let mut chosen_move = None;
    let mut max_value = i32::MIN;
    let mut total_visited = 0;

    for next_board in board.possible_moves(Player::Max) {
        let (value, visited) = evaluate_next(&next_board);
        if value > max_value {
            max_value = value;
            chosen_move = Some(next_board);
        }
        total_visited += visited;
    }

    if chosen_move == None {
        max_value = board.heuristic_evaluate();
        total_visited = 1;
    }

    return (chosen_move, max_value, total_visited);
}
