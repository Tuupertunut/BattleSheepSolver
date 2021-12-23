mod board;

#[cfg(test)]
mod tests;

use board::*;
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
        let (next_board, val, visited) = choose_move(player, &board, 6, i32::MIN + 1, i32::MAX);
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
                println!("(average turn took {:?})", total_duration / turns);

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

/* Minimax algorithm with alpha-beta pruning. This form is also called negamax. This function
 * returns the best move that the player can play, its value, and how many boards have been
 * evaluated. */
fn choose_move(
    player: Player,
    board: &Board,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (Option<Board>, i32, u64) {
    /* At depth 0 use heuristic evaluation. */
    if heuristic_depth == 0 {
        let chosen_move = None;
        let max_value = player.sign() * board.heuristic_evaluate();
        let total_visited = 1;
        return (chosen_move, max_value, total_visited);
    } else {
        /* At other depths find the best move by iterating through all moves. */
        let result;
        if heuristic_depth > 1 {
            /* Collect all moves into a vec and sort them before iterating them. Sort them by their
             * heuristic value so that moves with a better heuristic value are processed first. This
             * will cause alpha-beta pruning to kick in sooner. */
            let mut moves = board.possible_moves(player).collect::<Vec<Board>>();
            /* Min's moves are sorted smallest heuristic first and Max's by largest first. */
            moves.sort_by_cached_key(|next_board| -player.sign() * next_board.heuristic_evaluate());

            let iter = moves.into_iter();
            result = best_move(player, iter, heuristic_depth, alpha, beta);
        } else {
            /* Moves generated at depth 1 will only be evaluated by the heuristic, so they don't
             * need to be sorted. Just iterate the moves. */
            let iter = board.possible_moves(player);
            result = best_move(player, iter, heuristic_depth, alpha, beta);
        };
        let (chosen_move, max_value, total_visited) = result;

        /* If there were no possible moves, fall back to heuristic evaluation. */
        if chosen_move == None {
            let max_value = player.sign() * board.heuristic_evaluate();
            let total_visited = 1;
            return (chosen_move, max_value, total_visited);
        } else {
            return (chosen_move, max_value, total_visited);
        }
    }
}

/* Helper function used by choose_move that takes an iterator. This needs to be a separate function
 * so it can take any type of iterator. */
fn best_move<I: Iterator<Item = Board>>(
    player: Player,
    moves: I,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (Option<Board>, i32, u64) {
    let next_player = match player {
        Player::Min => Player::Max,
        Player::Max => Player::Min,
    };

    let mut chosen_move = None;
    let mut max_value = i32::MIN;
    let mut total_visited = 0;

    let mut alpha = alpha;

    /* Choosing the maximum value move. The moves are evaluated using minimax recursively on them. */
    for next_board in moves {
        /* This move is evaluated by the opposite player. For that reason both the alpha and beta
         * bounds and the resulting value are negated. This allows us to use the same function for
         * both players. */
        let (_, val, visited) =
            choose_move(next_player, &next_board, heuristic_depth - 1, -beta, -alpha);
        let value = -val;

        total_visited += visited;
        if value > max_value {
            max_value = value;
            chosen_move = Some(next_board);

            /* Alpha-beta pruning: If the value goes higher than beta, it means that
             * the caller of this function is not interested in this branch, so we can return early. */
            if max_value >= beta {
                return (chosen_move, max_value, total_visited);
            }
            /* Now that we have a value of at least max_value, we can increase alpha to signal that
             * we are not interested in child branches that produce a lower value. */
            alpha = i32::max(alpha, max_value);
        }
    }

    return (chosen_move, max_value, total_visited);
}
