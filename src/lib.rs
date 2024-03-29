pub mod board;

#[cfg(test)]
mod tests;

use board::{Board, Player};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Mutex,
};

pub fn sort_iter_by_cached_key<I, T, F, K>(iter: I, f: F) -> impl Iterator<Item = T>
where
    I: Iterator<Item = T>,
    F: FnMut(&T) -> K,
    K: Ord,
{
    let mut vec = iter.collect::<Vec<T>>();
    vec.sort_by_cached_key(f);
    return vec.into_iter();
}

/* Minimax algorithm functions. This variant of minimax is using alpha-beta pruning, move ordering
 * and parallelization to optimize its performance. It is also organized in a way called negamax,
 * where both Min and Max use the same evaluation function. */

/* Chooses the best next move for a player. Returns the next board, its value, and how many boards
 * have been evaluated. */
pub fn choose_move(
    player: Player,
    board: &Board,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (Option<Board>, i32, u64) {
    /* Sort all moves before iterating them. Sort them by their heuristic value so that moves with a
     * better heuristic value are processed first. This will cause alpha-beta pruning to take effect
     * sooner.
     * Min's moves are sorted smallest heuristic first and Max's by largest first. */
    let mut moves = sort_iter_by_cached_key(board.possible_moves(player), |next_board| {
        -player.direction() * next_board.heuristic_evaluate()
    });

    /* Result is wrapped in a mutex so it can be updated from multiple threads. */
    let result = Mutex::new((None, i32::MIN, 0));
    /* Alpha is an atomic integer so it can be accessed from multiple threads. It is not wrapped in
     * the same mutex as result, because it is accessed more often. */
    let alpha = AtomicI32::new(alpha);

    /* Closure that will be executed in the thread pool. */
    let evaluate_in_thread = |next_board| {
        /* This move is evaluated by the opposite player. For that reason both the alpha and beta
         * bounds and the resulting value are negated. This allows us to use the same function for
         * both players. */
        let (val, visited) = evaluate(
            player.next(),
            &next_board,
            heuristic_depth - 1,
            -beta,
            -alpha.load(Ordering::SeqCst),
        );
        let value = -val;

        /* Mutex is locked here. We can now update result. */
        let (chosen_move, max_value, total_visited) = &mut *result.lock().unwrap();

        *total_visited += visited;
        if value > *max_value {
            *max_value = value;
            *chosen_move = Some(next_board);

            /* Now that we have a value of at least max_value, we can increase alpha to signal that
             * we are not interested in child branches that produce a lower value. */
            alpha.fetch_max(*max_value, Ordering::SeqCst);
        }
        /* Mutex is unlocked here. */
    };

    /* Evaluate the first move before starting the parallel evaluation. This is called the Young
     * Brothers Wait Concept optimization. It ensures that all parallel evaluation jobs have a good
     * alpha value to start with. */
    if let Some(next_board) = moves.next() {
        evaluate_in_thread(next_board);
    }

    /* Parallelization: Instead of evaluating moves one by one, spawn an evaluation job into a
     * thread pool for each move. Then wait until all jobs spawned inside this scope are completed. */
    rayon::scope_fifo(|s| {
        for next_board in moves {
            s.spawn_fifo(|_| evaluate_in_thread(next_board));
        }
    });

    let (chosen_move, max_value, total_visited) = result.into_inner().unwrap();

    /* If there were no possible moves, fall back to heuristic evaluation. */
    if max_value == i32::MIN {
        let chosen_move = None;
        let max_value = player.direction() * board.heuristic_evaluate();
        let total_visited = 1;
        return (chosen_move, max_value, total_visited);
    }

    return (chosen_move, max_value, total_visited);
}

/* Evaluates a board either by heuristic or minimax. */
pub fn evaluate(
    player: Player,
    board: &Board,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (i32, u64) {
    /* At depth 0 use heuristic evaluation. */
    if heuristic_depth == 0 {
        let max_value = player.direction() * board.heuristic_evaluate();
        let total_visited = 1;
        return (max_value, total_visited);
    } else {
        /* At other depths use minimax evaluation. Minimax evaluation iterates through possible next
         * moves. */
        let result;
        if heuristic_depth > 1 {
            /* Sort all moves before iterating them. Sort them by their heuristic value so that
             * moves with a better heuristic value are processed first. This will cause alpha-beta
             * pruning to take effect sooner.
             * Min's moves are sorted smallest heuristic first and Max's by largest first. */
            let moves = sort_iter_by_cached_key(board.possible_moves(player), |next_board| {
                -player.direction() * next_board.heuristic_evaluate()
            });
            result = minimax_evaluate(player, moves, heuristic_depth, alpha, beta);
        } else {
            /* Moves generated at depth 1 will only be evaluated by the heuristic, so they don't
             * need to be sorted. Just iterate the moves. */
            let moves = board.possible_moves(player);
            result = minimax_evaluate(player, moves, heuristic_depth, alpha, beta);
        }
        let (max_value, total_visited) = result;

        /* If there were no possible moves, fall back to heuristic evaluation. */
        if max_value == i32::MIN {
            let max_value = player.direction() * board.heuristic_evaluate();
            let total_visited = 1;
            return (max_value, total_visited);
        }

        return (max_value, total_visited);
    }
}

/* Evaluates an iterator of moves by finding the move with the highest value. This function calls
 * evaluate() on the move boards, which may recursively call this function again. */
pub fn minimax_evaluate<I: Iterator<Item = Board>>(
    player: Player,
    moves: I,
    heuristic_depth: u32,
    alpha: i32,
    beta: i32,
) -> (i32, u64) {
    let mut max_value = i32::MIN;
    let mut total_visited = 0;

    let mut alpha = alpha;

    /* Finding the move with the largest value. */
    for next_board in moves {
        /* This move is evaluated by the opposite player. For that reason both the alpha and beta
         * bounds and the resulting value are negated. This allows us to use the same function for
         * both players. */
        let (val, visited) = evaluate(
            player.next(),
            &next_board,
            heuristic_depth - 1,
            -beta,
            -alpha,
        );
        let value = -val;

        total_visited += visited;
        if value > max_value {
            max_value = value;

            /* Alpha-beta pruning: If the value goes higher than beta, it means that
             * the caller of this function is not interested in this branch, so we can return early. */
            if max_value >= beta {
                return (max_value, total_visited);
            }
            /* Now that we have a value of at least max_value, we can increase alpha to signal that
             * we are not interested in child branches that produce a lower value. */
            alpha = i32::max(alpha, max_value);
        }
    }

    return (max_value, total_visited);
}
