use std::collections::HashSet;
use std::error::Error;
use std::iter;
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
pub struct Board {
    /* Tiles stored in row-major order. */
    tiles: Vec<Tile>,
    row_length: usize,
}

impl Index<(usize, usize)> for Board {
    type Output = Tile;

    fn index(&self, (r, q): (usize, usize)) -> &Self::Output {
        /* r = row coordinate, q = column coordinate
         * Return the tile for all valid coords in the board, but also return NoTile for all coords
         * outside the board. This way the indexing operation never panics. */
        if r < self.num_rows() && q < self.row_length {
            return &self.tiles[self.row_length * r + q];
        } else {
            return &Tile::NoTile;
        }
    }
}

impl IndexMut<(usize, usize)> for Board {
    fn index_mut(&mut self, (r, q): (usize, usize)) -> &mut Self::Output {
        return &mut self.tiles[self.row_length * r + q];
    }
}

impl Board {
    pub fn num_rows(&self) -> usize {
        return self.tiles.len() / self.row_length;
    }

    /* Iterates through all tiles in row-major order. */
    pub fn iter_row_major(&self) -> impl Iterator<Item = ((usize, usize), &Tile)> {
        return self
            .tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| ((index / self.row_length, index % self.row_length), tile));
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = (usize, &[Tile])> {
        return self.tiles.chunks_exact(self.row_length).enumerate();
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
        let row_strings = input
            .split("\n")
            /* Filter out whitespace-only rows. */
            .filter(|&row_string| !row_string.trim().is_empty())
            .enumerate()
            /* Indent each row so that the hexagonal grid becomes a square grid. The first row needs
             * to be indented by 0 spaces, the second by 2 spaces and so on. */
            .map(|(i, row_string)| {
                let indentation = i * 2;
                let row_indent = iter::repeat(' ').take(indentation).collect::<String>();
                return row_indent + row_string.trim_end();
            })
            .collect::<Vec<String>>();

        /* Column index of first board character in any row. */
        let string_begin_index = row_strings
            .iter()
            .map(|row_string| row_string.chars().take_while(|&char| char == ' ').count())
            .min()
            .unwrap_or(0)
            / 2
            * 2;
        /* Max number of tiles in any row. */
        let row_length = (row_strings
            .iter()
            .map(|row_string| row_string.len())
            .max()
            .unwrap_or(0)
            - string_begin_index
            + 3)
            / 4;
        /* Column index of last board character in any row. */
        let string_end_index = row_length * 4 + string_begin_index;

        let mut tiles = Vec::<Tile>::with_capacity(row_length * row_strings.len());

        for row_string in row_strings.iter() {
            /* The part of the row from begin index to end index, padded with spaces if needed. */
            let row_content = row_string
                .chars()
                .chain(iter::repeat(' '))
                .take(string_end_index)
                .skip(string_begin_index)
                .collect::<String>();

            /* Splitting row into 4 character pieces. */
            for tile_string in row_content
                .as_bytes()
                .chunks(4)
                .map(String::from_utf8_lossy)
            {
                let tile_content = tile_string.trim_end();

                if tile_content == "" {
                    tiles.push(Tile::NoTile);
                } else if tile_content == " 0" {
                    tiles.push(Tile::Empty);
                } else if tile_content.starts_with("+") {
                    let stack_size = tile_content[1..].parse::<u8>()?;
                    tiles.push(Tile::Stack(Player::Max, stack_size));
                } else if tile_content.starts_with("-") {
                    let stack_size = tile_content[1..].parse::<u8>()?;
                    tiles.push(Tile::Stack(Player::Min, stack_size));
                } else {
                    return Err("Invalid tile")?;
                }
            }
        }

        return Ok(Board { tiles, row_length });
    }

    /* Writes a board into a hexagonal board string. */
    pub fn write(&self, colored: bool) -> String {
        /* Ansi escape sequences for terminal colors. A colored text starts with a color sequence
         * and ends with a reset sequence. */
        const GREEN: &str = "\u{001b}[32m";
        const RED: &str = "\u{001b}[31;1m";
        const BLUE: &str = "\u{001b}[34;1m";
        const RESET: &str = "\u{001b}[0m";

        let mut row_strings = Vec::<String>::new();

        for (r, row) in self.iter_rows() {
            let mut row_string = String::new();

            /* Indent each row so that the string looks like a hexagonal grid. The last row needs to
             * be indented by 0 spaces, the second last by 2 spaces and so on. */
            let indentation = (self.num_rows() - 1 - r) * 2;
            let row_indent = iter::repeat(' ').take(indentation).collect::<String>();
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

        /* Column index of first board character in any row. */
        let string_begin_index = row_strings
            .iter()
            .map(|row_string| row_string.chars().take_while(|&char| char == ' ').count())
            .min()
            .unwrap_or(0)
            / 2
            * 2;

        /* Remove any unnecessary indentation and leading whitespace. */
        for row_string in row_strings.iter_mut() {
            *row_string = row_string[string_begin_index..].trim_end().to_string();
        }

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
     * by their neighbors. In the endgame, another heuristic is used. */
    pub fn heuristic_evaluate(&self) -> i32 {
        let mut value = 0;
        let mut max_all_blocked = true;
        let mut min_all_blocked = true;
        let mut max_stacks = 0;
        let mut min_stacks = 0;

        for (coords, &tile) in self.iter_row_major() {
            if let Tile::Stack(player, size) = tile {
                /* A maximum of 6 directions are blocked. */
                let mut blocked_directions = 6;
                for neighbor_coords in Board::iter_neighbor_coords(coords) {
                    if self[neighbor_coords] == Tile::Empty {
                        blocked_directions -= 1;
                    }
                }

                if size > 1 && blocked_directions < 6 {
                    match player {
                        Player::Min => min_all_blocked = false,
                        Player::Max => max_all_blocked = false,
                    }
                }

                /* Being surrounded from more sides and having more sheep in the stack increase
                 * its blocked score. */
                let blocked_score = (size as i32 - 1) * blocked_directions;

                /* A blocked Min stack gives an advantage to Max and therefore increases the
                 * value of the board. Vice versa for Max. */
                match player {
                    Player::Min => {
                        value += blocked_score;
                        min_stacks += 1;
                    }
                    Player::Max => {
                        value -= blocked_score;
                        max_stacks += 1;
                    }
                }
            }
        }

        /* If at least on player is blocked, use end game evaluation instead.
         *
         * Both players are blocked, so the game is over and the winner can be determined. */
        if min_all_blocked && max_all_blocked {
            if max_stacks > min_stacks {
                value = 1000000;
            } else if min_stacks > max_stacks {
                value = -1000000;
            } else {
                value = match self.largest_connected_field_holder() {
                    Some(Player::Max) => 1000000,
                    Some(Player::Min) => -1000000,
                    None => 0,
                }
            }
        /* Only one player is blocked. In most cases this means that the blocked player has lost. In
         * the rare case that the beginning player has blocked themselves, there is a chance that
         * they might still win. */
        } else if min_all_blocked {
            if max_stacks >= min_stacks {
                value = 1000000;
            } else {
                /* The rare case where the blocked player might still win. However, if the other
                 * player already has a larger connected field, the blocked player will lose. If
                 * not, the game is not yet over and we fall back to the normal heuristic
                 * evaluation. */
                if let Some(Player::Max) = self.largest_connected_field_holder() {
                    value = 1000000;
                }
            }
        } else if max_all_blocked {
            if min_stacks >= max_stacks {
                value = -1000000;
            } else {
                if let Some(Player::Min) = self.largest_connected_field_holder() {
                    value = -1000000;
                }
            }
        }

        return value;
    }

    /* Tells which player has the largest connected field. */
    pub fn largest_connected_field_holder(&self) -> Option<Player> {
        let mut min_largest_field = 0;
        let mut max_largest_field = 0;

        let mut visited = HashSet::<(usize, usize)>::new();
        let mut dfs_stack = Vec::<(usize, usize)>::new();
        for (start_coords, &tile) in self.iter_row_major() {
            if let Tile::Stack(player, _) = tile {
                if !visited.contains(&start_coords) {
                    let mut field_size = 0;

                    /* Depth-first search for counting the size of a connected field. */
                    visited.insert(start_coords);
                    dfs_stack.push(start_coords);
                    while let Some(coords) = dfs_stack.pop() {
                        field_size += 1;

                        for neighbor_coords in Board::iter_neighbor_coords(coords) {
                            if let Tile::Stack(neighbor_player, _) = self[neighbor_coords] {
                                if neighbor_player == player && !visited.contains(&neighbor_coords)
                                {
                                    visited.insert(neighbor_coords);
                                    dfs_stack.push(neighbor_coords);
                                }
                            }
                        }
                    }

                    match player {
                        Player::Min => {
                            min_largest_field = u32::max(min_largest_field, field_size);
                        }
                        Player::Max => {
                            max_largest_field = u32::max(max_largest_field, field_size);
                        }
                    }
                }
            }
        }
        return if min_largest_field > max_largest_field {
            Some(Player::Min)
        } else if max_largest_field > min_largest_field {
            Some(Player::Max)
        } else {
            None
        };
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
