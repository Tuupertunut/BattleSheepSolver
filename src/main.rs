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

        /* Here we tell each player how to choose the next turn. */
        let (next_board, value, visited) = match player {
            Player::Min => min_choose(&board, 5, i32::MIN, i32::MAX),
            Player::Max => max_choose(&board, 5, i32::MIN, i32::MAX),
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

/* Minimax algorithm with alpha-beta pruning. */
fn min_choose(
    board: &Board,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (Option<Board>, i32, u64) {
    let mut chosen_move = None;
    let mut min_value = i32::MAX;
    let mut total_visited = 0;

    /* At depth 0 use heuristic evaluation. */
    if heuristic_depth == 0 {
        min_value = board.heuristic_evaluate();
        total_visited = 1;
    } else {
        let mut beta = beta;

        /* Choose the minimum value move. */
        for next_board in board.possible_moves(Player::Min) {
            let (_, value, visited) = max_choose(&next_board, heuristic_depth - 1, alpha, beta);

            total_visited += visited;
            if value < min_value {
                min_value = value;
                chosen_move = Some(next_board);

                /* Alpha-beta pruning: If the value goes lower than alpha, there is no chance that
                 * max would ever choose this branch, so we can return early. */
                if min_value <= alpha {
                    return (chosen_move, min_value, total_visited);
                }
                beta = i32::min(beta, min_value);
            }
        }

        /* If there were no possible moves, fall back to heuristic evaluation. */
        if chosen_move == None {
            min_value = board.heuristic_evaluate();
            total_visited = 1;
        }
    }

    return (chosen_move, min_value, total_visited);
}

fn max_choose(
    board: &Board,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (Option<Board>, i32, u64) {
    let mut chosen_move = None;
    let mut max_value = i32::MIN;
    let mut total_visited = 0;

    if heuristic_depth == 0 {
        max_value = board.heuristic_evaluate();
        total_visited = 1;
    } else {
        let mut alpha = alpha;

        for next_board in board.possible_moves(Player::Max) {
            let (_, value, visited) = min_choose(&next_board, heuristic_depth - 1, alpha, beta);

            total_visited += visited;
            if value > max_value {
                max_value = value;
                chosen_move = Some(next_board);

                if max_value >= beta {
                    return (chosen_move, max_value, total_visited);
                }
                alpha = i32::max(alpha, max_value);
            }
        }

        if chosen_move == None {
            max_value = board.heuristic_evaluate();
            total_visited = 1;
        }
    }

    return (chosen_move, max_value, total_visited);
}
