use std::error::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Player {
    Min,
    Max,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Tile {
    NoTile, /* outside of the board */
    Empty,
    Stack(Player, u8),
}

/* Coordinate offsets for each neighbor in a hex grid. Neighbors can be found by adding these to our
 * current coordinates. These also represent straight line directions. */
pub const NEIGHBOR_OFFSETS: [(isize, isize); 6] =
    [(0, 1), (1, 1), (1, 0), (0, -1), (-1, -1), (-1, 0)];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Board(Vec<Vec<Tile>>);

impl Board {
    pub fn parse(input: &str) -> Result<Board, Box<dyn Error>> {
        let mut board = Vec::<Vec<Tile>>::new();

        for row_string in input.trim_end().split("\n") {
            let mut row = Vec::<Tile>::new();

            /* Splitting row into 4 character pieces. */
            for tile_string in row_string
                .trim_end()
                .as_bytes()
                .chunks(4)
                .map(String::from_utf8_lossy)
            {
                let tile_content = tile_string.trim_end();

                if tile_content == "" {
                    row.push(Tile::NoTile);
                } else if tile_content == " 0" {
                    row.push(Tile::Empty);
                } else if tile_content.starts_with("+") {
                    let stack_size = tile_content[1..].parse::<u8>()?;
                    row.push(Tile::Stack(Player::Max, stack_size));
                } else if tile_content.starts_with("-") {
                    let stack_size = tile_content[1..].parse::<u8>()?;
                    row.push(Tile::Stack(Player::Min, stack_size));
                } else {
                    return Err("Invalid tile")?;
                }
            }

            board.push(row);
        }

        return Ok(Board(board));
    }

    pub fn write(&self) -> String {
        let Board(board) = self;

        let mut output = String::new();

        for row in board {
            for &tile in row {
                let tile_string = match tile {
                    Tile::NoTile => format!("    "),
                    Tile::Empty => format!(" 0  "),
                    Tile::Stack(Player::Max, stack_size) => format!("+{:<3}", stack_size),
                    Tile::Stack(Player::Min, stack_size) => format!("-{:<3}", stack_size),
                };
                output.push_str(&tile_string);
            }
            output.push_str("\n");
        }

        return output;
    }

    pub fn possible_moves(&self, player: Player) -> Vec<Board> {
        let Board(board) = self;

        let mut next_boards = Vec::<Board>::new();

        /* r = row coordinate, q = column coordinate */
        for (from_r, row) in board.iter().enumerate() {
            for (from_q, &tile) in row.iter().enumerate() {
                if let Tile::Stack(p, size) = tile {
                    if p == player && size > 1 {
                        /* Loop through all straight line directions. */
                        for (dir_r, dir_q) in NEIGHBOR_OFFSETS {
                            /* Move to a direction as far as there are empty tiles. */
                            let mut r = from_r;
                            let mut q = from_q;
                            loop {
                                /* Coordinates for the next tile in the direction.
                                 * Hack: negative numbers cannot be added to a usize, so they are
                                 * converted into usize with underflow and then added with overflow.
                                 * Same as: let next_r = r + dir_r */
                                let next_r = r.wrapping_add(dir_r as usize);
                                let next_q = q.wrapping_add(dir_q as usize);

                                /* If next tile is empty, move to that tile. */
                                if next_r < board.len()
                                    && next_q < board[next_r].len()
                                    && board[next_r][next_q] == Tile::Empty
                                {
                                    r = next_r;
                                    q = next_q;
                                } else {
                                    break;
                                }
                            }

                            /* Check if we actually found any empty tiles in the direction. */
                            if r != from_r || q != from_q {
                                for split in 1..size {
                                    let mut next_board = board.clone();
                                    next_board[r][q] = Tile::Stack(player, split);
                                    next_board[from_r][from_q] = Tile::Stack(player, size - split);
                                    next_boards.push(Board(next_board));
                                }
                            }
                        }
                    }
                }
            }
        }

        return next_boards;
    }

    /* Evaluates the current board state. Positive number means Max has an advantage, negative means
     * Min has it. This is a very simple evaluation function that checks how blocked the stacks are
     * by their neighbors. */
    pub fn evaluate(&self) -> i32 {
        let Board(board) = self;

        let mut value = 0;
        for (r, row) in board.iter().enumerate() {
            for (q, &tile) in row.iter().enumerate() {
                if let Tile::Stack(player, size) = tile {
                    /* A maximum of 6 directions are blocked. */
                    let mut blocked_directions = 6;
                    for (offset_r, offset_q) in NEIGHBOR_OFFSETS {
                        let neighbor_r = r.wrapping_add(offset_r as usize);
                        let neighbor_q = q.wrapping_add(offset_q as usize);
                        if neighbor_r < board.len()
                            && neighbor_q < board[neighbor_r].len()
                            && board[neighbor_r][neighbor_q] == Tile::Empty
                        {
                            blocked_directions -= 1;
                        }
                    }

                    /* Being surrounded from more sides and having more sheep in the stack increase
                     * its blocked score. */
                    let blocked_score = (size as i32 - 1) * blocked_directions;

                    /* A blocked Min stack gives an advantage to Max and therefore increases the
                     * value of the board. Vice versa for Max. */
                    match player {
                        Player::Min => value += blocked_score,
                        Player::Max => value -= blocked_score,
                    }
                }
            }
        }
        return value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn output_equals_input() {
        /* Multiline strings are not indented correctly because the indentation would change the
         * string content. */
        let input = &"
 0  +2  
-2   0  -3  +3  
     0           0  
"[1..];
        assert_eq!(input, Board::parse(input).unwrap().write());
    }

    #[test]
    fn parse_fails_on_invalid_board() {
        assert!(Board::parse("abcdefg").is_err());
    }

    #[test]
    fn possible_moves_are_found() {
        let input = &"
 0  +2  
-2   0  -3  +3  
     0           0  
"[1..];
        let max_moves = [
            &"
+1  +1  
-2   0  -3  +3  
     0           0  
"[1..],
            &"
 0  +1  
-2   0  -3  +3  
    +1           0  
"[1..],
            &"
 0  +2  
-2   0  -3  +2  
     0          +1  
"[1..],
            &"
 0  +2  
-2   0  -3  +1  
     0          +2  
"[1..],
        ];
        assert_eq!(
            Board::parse(input)
                .unwrap()
                .possible_moves(Player::Max)
                .into_iter()
                .collect::<HashSet<Board>>(),
            max_moves
                .iter()
                .map(|s| Board::parse(s).unwrap())
                .collect::<HashSet<Board>>()
        );
    }
}
