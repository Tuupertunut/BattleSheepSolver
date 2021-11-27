use std::collections::HashSet;
use std::error::Error;
use std::ops::{Index, IndexMut};

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

/* Helper function used by Board::parse and Board::write. */
fn unindent(rows: Vec<String>) -> Vec<String> {
    let indentation = rows
        .iter()
        .map(|row| row.chars().take_while(|&char| char == ' ').count())
        .min()
        .unwrap_or(0);
    let even_indentation = indentation / 2 * 2;
    let unindented = rows
        .iter()
        .map(|row| row[even_indentation..].to_string())
        .collect::<Vec<String>>();
    return unindented;
}

impl Index<(usize, usize)> for Board {
    type Output = Tile;

    fn index(&self, (r, q): (usize, usize)) -> &Self::Output {
        let Board(board) = self;

        /* r = row coordinate, q = column coordinate
         * Return the tile for all valid coords in the board, but also return NoTile for all coords
         * outside the board. This way the indexing operation never panics. */
        if r < board.len() && q < board[r].len() {
            return &board[r][q];
        } else {
            return &Tile::NoTile;
        }
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (r, q): (usize, usize)) -> &mut Self::Output {
        let Board(board) = self;

        return &mut board[r][q];
    }
}

impl Board {
    /* Iterates through all tiles in row-major order. */
    pub fn iter_row_major(&self) -> impl Iterator<Item = ((usize, usize), &Tile)> {
        let Board(board) = self;

        return board
            .iter()
            .enumerate()
            .flat_map(|(r, row)| row.iter().enumerate().map(move |(q, tile)| ((r, q), tile)));
    }

    /* Iterates through all neighbors of the given coordinates. */
    pub fn iter_neighbor_coords((r, q): (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
        return NEIGHBOR_OFFSETS.iter().map(move |&(offset_r, offset_q)| {
            /* Hack: negative numbers cannot be added to a usize, so they are converted into usize
             * with underflow and then added with overflow.
             * Same as:
             * r + offset_r
             * q + offset_q */
            (
                r.wrapping_add(offset_r as usize),
                q.wrapping_add(offset_q as usize),
            )
        });
    }

    /* Parses a hexagonal grid string into a board. */
    pub fn parse(input: &str) -> Result<Board, Box<dyn Error>> {
        let mut row_strings = input
            .split("\n")
            /* Filter out whitespace-only rows. */
            .filter(|&row_string| !row_string.trim().is_empty())
            .enumerate()
            /* Indent each row so that the hexagonal grid becomes a square grid. The first row needs
             * to be indented by 0 spaces, the second by 2 spaces and so on. */
            .map(|(i, row_string)| {
                let indentation = i * 2;
                let row_indent = std::iter::repeat(' ').take(indentation).collect::<String>();
                return row_indent + row_string.trim_end();
            })
            .collect::<Vec<String>>();
        /* Remove unnecessary indentation. */
        row_strings = unindent(row_strings);

        let mut board = Vec::<Vec<Tile>>::new();

        for row_string in row_strings {
            let mut row = Vec::<Tile>::new();

            /* Splitting row into 4 character pieces. */
            for tile_string in row_string.as_bytes().chunks(4).map(String::from_utf8_lossy) {
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

    /* Writes a board into a hexagonal board string. */
    pub fn write(&self, colored: bool) -> String {
        let Board(board) = self;

        /* Ansi escape sequences for terminal colors. A colored text starts with a color sequence
         * and ends with a reset sequence. */
        const GREEN: &str = "\u{001b}[32m";
        const RED: &str = "\u{001b}[31;1m";
        const BLUE: &str = "\u{001b}[34;1m";
        const RESET: &str = "\u{001b}[0m";

        let mut row_strings = Vec::<String>::new();

        for (i, row) in board.iter().enumerate() {
            let mut row_string = String::new();

            /* Indent each row so that the string looks like a hexagonal grid. The last row needs to
             * be indented by 0 spaces, the second last by 2 spaces and so on. */
            let indentation = (board.len() - 1 - i) * 2;
            let row_indent = std::iter::repeat(' ').take(indentation).collect::<String>();
            row_string.push_str(&row_indent);

            for &tile in row {
                let tile_string = if colored {
                    match tile {
                        Tile::NoTile => format!("    "),
                        Tile::Empty => format!("{} 0  {}", GREEN, RESET),
                        Tile::Stack(Player::Max, stack_size) => {
                            format!("{}+{:<3}{}", BLUE, stack_size, RESET)
                        }
                        Tile::Stack(Player::Min, stack_size) => {
                            format!("{}-{:<3}{}", RED, stack_size, RESET)
                        }
                    }
                } else {
                    match tile {
                        Tile::NoTile => format!("    "),
                        Tile::Empty => format!(" 0  "),
                        Tile::Stack(Player::Max, stack_size) => format!("+{:<3}", stack_size),
                        Tile::Stack(Player::Min, stack_size) => format!("-{:<3}", stack_size),
                    }
                };
                row_string.push_str(&tile_string);
            }

            row_strings.push(row_string);
        }

        /* Remove unnecessary indentation. */
        row_strings = unindent(row_strings);

        let output = row_strings.join("\n");
        return output;
    }

    /* Computes all possible next moves for a player. */
    pub fn possible_moves(&self, player: Player) -> Vec<Board> {
        let mut next_boards = Vec::<Board>::new();

        for (orig_coords, &tile) in self.iter_row_major() {
            if let Tile::Stack(p, size) = tile {
                if p == player && size > 1 {
                    /* Loop through all straight line directions. */
                    for dir_offset in NEIGHBOR_OFFSETS {
                        /* Move to a direction as far as there are empty tiles. */
                        let mut coords = orig_coords;
                        loop {
                            /* Coordinates for the next tile in the direction.
                             * Hack: negative numbers cannot be added to a usize, so they are
                             * converted into usize with underflow and then added with overflow.
                             * Same as: let next_coords = coords + dir_offset */
                            let next_coords = (
                                coords.0.wrapping_add(dir_offset.0 as usize),
                                coords.1.wrapping_add(dir_offset.1 as usize),
                            );

                            /* If next tile is empty, move to that tile. */
                            if self[next_coords] == Tile::Empty {
                                coords = next_coords;
                            } else {
                                break;
                            }
                        }

                        /* Check if we actually found any empty tiles in the direction. */
                        if coords != orig_coords {
                            for split in 1..size {
                                let mut next_board = self.clone();
                                next_board[coords] = Tile::Stack(player, split);
                                next_board[orig_coords] = Tile::Stack(player, size - split);
                                next_boards.push(next_board);
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
    pub fn heuristic_evaluate(&self) -> i32 {
        let mut value = 0;
        for (coords, &tile) in self.iter_row_major() {
            if let Tile::Stack(player, size) = tile {
                /* A maximum of 6 directions are blocked. */
                let mut blocked_directions = 6;
                for neighbor_coords in Board::iter_neighbor_coords(coords) {
                    if self[neighbor_coords] == Tile::Empty {
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
        return value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_equals_input() {
        /* Multiline strings are not indented correctly because the indentation would change the
         * string content. */
        let input = "
   0  +2  
-2   0  -3  +3  
   0           0  
"
        .trim_matches('\n');
        assert_eq!(input, Board::parse(input).unwrap().write(false));
    }

    #[test]
    fn parse_fails_on_invalid_board() {
        assert!(Board::parse("abcdefg").is_err());
    }

    #[test]
    fn possible_moves_are_found() {
        let input = "
   0  +2  
-2   0  -3  +3  
   0           0  
"
        .trim_matches('\n');
        let max_moves = [
            "
  +1  +1  
-2   0  -3  +3  
   0           0  
"
            .trim_matches('\n'),
            "
   0  +1  
-2   0  -3  +3  
  +1           0  
"
            .trim_matches('\n'),
            "
   0  +2  
-2   0  -3  +2  
   0          +1  
"
            .trim_matches('\n'),
            "
   0  +2  
-2   0  -3  +1  
   0          +2  
"
            .trim_matches('\n'),
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

    #[test]
    fn win_evaluates_as_winners_advantage() {
        let max_wins = "
  +14 +1   0   0  
-15 +1  -1   0  
"
        .trim_matches('\n');
        assert!(Board::parse(max_wins).unwrap().heuristic_evaluate() > 0);
    }

    #[test]
    fn end_with_equal_controlled_tiles_considers_field_size() {
        let max_has_greater_field = "
  +15 -1   0   0  
-15 +1   0   0  
"
        .trim_matches('\n');
        assert!(
            Board::parse(max_has_greater_field)
                .unwrap()
                .heuristic_evaluate()
                > 0
        );
    }

    #[test]
    fn draw_evaluates_as_zero() {
        let draw = "
  +1   0  -1  +14  
-14 +1   0  -1  
"
        .trim_matches('\n');
        assert!(Board::parse(draw).unwrap().heuristic_evaluate() == 0);
    }

    #[test]
    fn in_end_tile_count_weighs_more_than_field_size() {
        let min_wins = "
             0   0  
  +8  -1   0  -1  
-14 +8  
"
        .trim_matches('\n');
        assert!(Board::parse(min_wins).unwrap().heuristic_evaluate() < 0);
    }

    #[test]
    fn win_evaluates_higher_than_continuing_game() {
        let min_wins = "
     0  
   0   0   0  
     0   0  
  -15 
+16 -1   0   0   0   0   0   0   0   0  
"
        .trim_matches('\n');
        let min_will_lose = "
     0  
   0  -15  0  
     0   0  
  -1  
+16  0   0   0   0   0   0   0   0   0  
"
        .trim_matches('\n');
        assert!(
            Board::parse(min_wins).unwrap().heuristic_evaluate()
                < Board::parse(min_will_lose).unwrap().heuristic_evaluate()
        );
    }
}
