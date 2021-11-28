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
        let (next_board, value, count) = match player {
            Player::Min => min_choose(&board, |board_1| {
                max_value(board_1, |board_2| {
                    min_value(board_2, |board_3| {
                        max_value(board_3, |board_4| (board_4.heuristic_evaluate(), 1))
                    })
                })
            }),
            Player::Max => max_choose(&board, |board_1| {
                min_value(board_1, |board_2| {
                    max_value(board_2, |board_3| {
                        min_value(board_3, |board_4| (board_4.heuristic_evaluate(), 1))
                    })
                })
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
                    count,
                    value,
                    next_board.write(true)
                );
                board = next_board;
            }
        }
    }
}

fn min_value<F>(board: &Board, max_value_function: F) -> (i32, u64)
where
    F: Fn(&Board) -> (i32, u64),
{
    let mut value = i32::MAX;
    let mut count = 0;
    for next_board in board.possible_moves(Player::Min) {
        let (next_board_value, next_board_count) = max_value_function(&next_board);
        value = i32::min(value, next_board_value);
        count += next_board_count;
    }
    if value == i32::MAX {
        value = board.heuristic_evaluate();
        count += 1;
    }
    return (value, count);
}

fn max_value<F>(board: &Board, min_value_function: F) -> (i32, u64)
where
    F: Fn(&Board) -> (i32, u64),
{
    let mut value = i32::MIN;
    let mut count = 0;
    for next_board in board.possible_moves(Player::Max) {
        let (next_board_value, next_board_count) = min_value_function(&next_board);
        value = i32::max(value, next_board_value);
        count += next_board_count;
    }
    if value == i32::MIN {
        value = board.heuristic_evaluate();
        count += 1;
    }
    return (value, count);
}

fn min_choose<F>(board: &Board, max_value_function: F) -> (Option<Board>, i32, u64)
where
    F: Fn(&Board) -> (i32, u64),
{
    let mut value = i32::MAX;
    let mut chosen_move = None;
    let mut count = 0;
    for next_board in board.possible_moves(Player::Min) {
        let (next_board_value, next_board_count) = max_value_function(&next_board);
        if next_board_value < value {
            value = next_board_value;
            chosen_move = Some(next_board);
        }
        count += next_board_count;
    }
    if value == i32::MAX {
        value = board.heuristic_evaluate();
        count += 1;
    }
    return (chosen_move, value, count);
}

fn max_choose<F>(board: &Board, min_value_function: F) -> (Option<Board>, i32, u64)
where
    F: Fn(&Board) -> (i32, u64),
{
    let mut value = i32::MIN;
    let mut chosen_move = None;
    let mut count = 0;
    for next_board in board.possible_moves(Player::Max) {
        let (next_board_value, next_board_count) = min_value_function(&next_board);
        if next_board_value > value {
            value = next_board_value;
            chosen_move = Some(next_board);
        }
        count += next_board_count;
    }
    if value == i32::MIN {
        value = board.heuristic_evaluate();
        count += 1;
    }
    return (chosen_move, value, count);
}
