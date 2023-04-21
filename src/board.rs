use either::Either;
use next_gen::prelude::*;
use std::{
    error::Error,
    iter,
    ops::{Index, IndexMut},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum Player {
    Min,
    Max,
}

impl Player {
    pub fn sign(self) -> i32 {
        match self {
            Self::Min => -1,
            Self::Max => 1,
        }
    }

    pub fn opposite(self) -> Player {
        match self {
            Player::Min => Player::Max,
            Player::Max => Player::Min,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum TileType {
    NoTile, /* outside of the board */
    Empty,
    Stack,
}

/* Custom bitfield struct for saving a Battle Sheep tile into a single byte.
 * Structure:
 * 2 bits tile_type, 00 = Stack, 01 = NoTile, 10 or 11 = Empty
 * 1 bits player, 0 = Min, 1 = Max
 * 5 bits stack_size
 * Numerically:
 * 0-31 = Min player's Stack with size 0-31
 * 32-63 = Max player's Stack with size 0-31
 * 64-127 = NoTile
 * 128-255 = Empty */
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Tile(u8);

impl Tile {
    pub const MAX_STACK_SIZE: u8 = 31;

    pub const NO_TILE: Self = Self::new(TileType::NoTile, Player::Min, 0);
    pub const EMPTY: Self = Self::new(TileType::Empty, Player::Min, 0);

    pub const fn new(tile_type: TileType, player: Player, stack_size: u8) -> Self {
        let bitfield = stack_size
            + match player {
                Player::Min => 0,
                Player::Max => 32,
            }
            + match tile_type {
                TileType::Stack => 0,
                TileType::NoTile => 64,
                TileType::Empty => 128,
            };
        return Self(bitfield);
    }

    pub fn tile_type(self) -> TileType {
        if self.0 < 64 {
            return TileType::Stack;
        } else if self.0 < 128 {
            return TileType::NoTile;
        } else {
            return TileType::Empty;
        }
    }

    pub fn player(self) -> Player {
        if self.0 < 32 {
            return Player::Min;
        } else {
            return Player::Max;
        }
    }

    pub fn stack_size(self) -> u8 {
        return self.0 % 32;
    }

    pub fn is_stack(self) -> bool {
        return self.0 < 64;
    }

    pub fn is_empty(self) -> bool {
        return self.0 >= 128;
    }

    pub fn is_board_tile(self) -> bool {
        return self.is_stack() || self.is_empty();
    }
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
            return &Tile::NO_TILE;
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
    pub fn iter_row_major(&self) -> impl Iterator<Item = ((usize, usize), Tile)> + '_ {
        return self
            .tiles
            .iter()
            .enumerate()
            .map(|(index, &tile)| ((index / self.row_length, index % self.row_length), tile));
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = (usize, &[Tile])> {
        return self.tiles.chunks_exact(self.row_length).enumerate();
    }

    /* Iterates through all neighbors of the given coordinates in clockwise direction. */
    pub fn iter_neighbors(
        &self,
        coords: (usize, usize),
    ) -> impl Iterator<Item = ((usize, usize), Tile)> + '_ {
        return NEIGHBOR_OFFSETS.iter().map(move |&offset| {
            /* Hack: negative numbers cannot be added to a usize, so they are converted into usize
             * with underflow and then added with overflow.
             * Same as: let neighbor_coords = coords + offset */
            let neighbor_coords = (
                coords.0.wrapping_add(offset.0 as usize),
                coords.1.wrapping_add(offset.1 as usize),
            );
            return (neighbor_coords, self[neighbor_coords]);
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

        if row_strings.is_empty() {
            return Err("Empty board")?;
        }

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
                    tiles.push(Tile::NO_TILE);
                } else if tile_content == " 0" {
                    tiles.push(Tile::EMPTY);
                } else if tile_content.starts_with("+") {
                    let stack_size = tile_content[1..].parse::<u8>()?;
                    if stack_size > Tile::MAX_STACK_SIZE {
                        return Err(format!("Stack size over {}", Tile::MAX_STACK_SIZE))?;
                    }
                    tiles.push(Tile::new(TileType::Stack, Player::Max, stack_size));
                } else if tile_content.starts_with("-") {
                    let stack_size = tile_content[1..].parse::<u8>()?;
                    if stack_size > Tile::MAX_STACK_SIZE {
                        return Err(format!("Stack size over {}", Tile::MAX_STACK_SIZE))?;
                    }
                    tiles.push(Tile::new(TileType::Stack, Player::Min, stack_size));
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
                let tile = (tile.tile_type(), tile.player(), tile.stack_size());
                let tile_string = if colored {
                    match tile {
                        (TileType::NoTile, _, _) => format!("    "),
                        (TileType::Empty, _, _) => format!("{} 0  {}", GREEN, RESET),
                        (TileType::Stack, Player::Max, stack_size) => {
                            format!("{}+{:<3}{}", BLUE, stack_size, RESET)
                        }
                        (TileType::Stack, Player::Min, stack_size) => {
                            format!("{}-{:<3}{}", RED, stack_size, RESET)
                        }
                    }
                } else {
                    match tile {
                        (TileType::NoTile, _, _) => format!("    "),
                        (TileType::Empty, _, _) => format!(" 0  "),
                        (TileType::Stack, Player::Max, stack_size) => {
                            format!("+{:<3}", stack_size)
                        }
                        (TileType::Stack, Player::Min, stack_size) => {
                            format!("-{:<3}", stack_size)
                        }
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

    /* Iterates through all possible next moves for a player. */
    pub fn possible_moves(&self, player: Player) -> impl Iterator<Item = Board> + '_ {
        let player_has_stacks = self
            .iter_row_major()
            .any(|(_, tile)| tile.is_stack() && tile.player() == player);

        if player_has_stacks {
            return Either::Right(self.possible_regular_moves(player));
        } else {
            return Either::Left(self.possible_starting_moves(player));
        }
    }

    /* Iterates through regular moves where player splits a stack and moves it. */
    fn possible_regular_moves(&self, player: Player) -> impl Iterator<Item = Board> + '_ {
        #[generator(Board)]
        fn generate_moves(board: &Board, player: Player) {
            /* Iterate through all tiles. */
            for (orig_coords, tile) in board.iter_row_major() {
                /* Check if the tile is a splittable stack of this player. */
                if tile.is_stack() && tile.player() == player {
                    let stack_size = tile.stack_size();
                    if stack_size > 1 {
                        /* Iterate through all straight line directions. */
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
                                if board[next_coords].is_empty() {
                                    coords = next_coords;
                                } else {
                                    break;
                                }
                            }
                            /* Check if we actually found any empty tiles in the direction. */
                            if coords != orig_coords {
                                /* Iterate through all the ways to split the stack. */
                                for split in 1..stack_size {
                                    let mut next_board = board.clone();
                                    next_board[coords] = Tile::new(TileType::Stack, player, split);
                                    next_board[orig_coords] =
                                        Tile::new(TileType::Stack, player, stack_size - split);

                                    yield_!(next_board);
                                }
                            }
                        }
                    }
                }
            }
        }

        mk_gen!(let generator = box generate_moves(self, player));
        return generator.into_iter();
    }

    /* Iterates through starting moves where player places a stack on the outer edge. */
    fn possible_starting_moves(&self, player: Player) -> impl Iterator<Item = Board> + '_ {
        #[generator(Board)]
        fn generate_moves(board: &Board, player: Player) {
            /* We know that the first board tile we encounter must be on the outer edge. */
            let (start_coords, start) = board
                .iter_row_major()
                .find(|&(_, tile)| tile.is_board_tile())
                .expect("The board is empty");

            /* The first board tile we encountered must be on the left edge of the board, so its
             * left side (offset (0, -1)) is a safe direction to start iterating neighbors. */
            let mut previous_coords = (start_coords.0, start_coords.1.wrapping_sub(1));
            let mut coords = start_coords;

            /* Iterate along the outer edge of the board. */
            loop {
                /* Search through the neighbors of coords in clockwise direction starting from
                 * previous_coords. Find the first board tile. We know that board tile is also on
                 * the outer edge. */
                let (next_coords, next) = board
                    .iter_neighbors(coords)
                    .chain(board.iter_neighbors(coords))
                    .skip_while(|&(neighbor_coords, _)| neighbor_coords != previous_coords)
                    .skip(1)
                    .find(|&(_, neighbor)| neighbor.is_board_tile())
                    .unwrap_or((start_coords, start));

                if next.is_empty() {
                    let mut next_board = board.clone();
                    next_board[next_coords] = Tile::new(TileType::Stack, player, 16);

                    yield_!(next_board);
                }

                /* We have come a full circle. */
                if next_coords == start_coords {
                    break;
                }

                previous_coords = coords;
                coords = next_coords;
            }
        }

        mk_gen!(let generator = box generate_moves(self, player));
        return generator.into_iter();
    }

    /* Evaluates the current board state. Positive number means Max has an advantage, negative means
     * Min has it. This is a very simple evaluation function that checks how blocked the stacks are
     * by their neighbors and how evenly split they are. In the endgame, another heuristic is used. */
    pub fn heuristic_evaluate(&self) -> i32 {
        let mut value = 0;
        let mut max_all_blocked = true;
        let mut min_all_blocked = true;
        let mut max_stacks = 0;
        let mut min_stacks = 0;

        let mut max_largest_stack = 0;
        let mut max_smallest_stack = i32::MAX;
        let mut min_largest_stack = 0;
        let mut min_smallest_stack = i32::MAX;

        for (coords, tile) in self.iter_row_major() {
            if tile.is_stack() {
                let player = tile.player();
                let size = tile.stack_size();

                /* A maximum of 6 directions are blocked. */
                let mut blocked_directions = 6;
                for (_, neighbor) in self.iter_neighbors(coords) {
                    if neighbor.is_empty() {
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

                match player {
                    Player::Min => {
                        /* A blocked Min stack gives an advantage to Max and therefore increases the
                         * value of the board. Vice versa for Max. */
                        value += blocked_score;
                        min_stacks += 1;
                        min_largest_stack = i32::max(min_largest_stack, size as i32);
                        min_smallest_stack = i32::min(min_smallest_stack, size as i32);
                    }
                    Player::Max => {
                        value -= blocked_score;
                        max_stacks += 1;
                        max_largest_stack = i32::max(max_largest_stack, size as i32);
                        max_smallest_stack = i32::min(max_smallest_stack, size as i32);
                    }
                }
            }
        }

        /* Extra score for splitting stacks evenly. This does not matter as much as being blocked,
         * the maximum splitting bonus is 7. */
        value += (min_largest_stack - min_smallest_stack) / 2;
        value -= (max_largest_stack - max_smallest_stack) / 2;

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

        /* Helper functions for using the visited array. */
        let is_visited = |visited: &Vec<bool>, (r, q)| visited[self.row_length * r + q];
        let set_visited = |visited: &mut Vec<bool>, (r, q)| visited[self.row_length * r + q] = true;

        let mut visited = vec![false; self.tiles.len()];
        let mut dfs_stack = Vec::<(usize, usize)>::new();

        for (start_coords, tile) in self.iter_row_major() {
            if tile.is_stack() && !is_visited(&visited, start_coords) {
                let player = tile.player();
                let mut field_size = 0;

                /* Depth-first search for counting the size of a connected field. */
                set_visited(&mut visited, start_coords);
                dfs_stack.push(start_coords);
                while let Some(coords) = dfs_stack.pop() {
                    field_size += 1;

                    for (neighbor_coords, neighbor) in self.iter_neighbors(coords) {
                        if neighbor.is_stack()
                            && neighbor.player() == player
                            && !is_visited(&visited, neighbor_coords)
                        {
                            set_visited(&mut visited, neighbor_coords);
                            dfs_stack.push(neighbor_coords);
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
        return if min_largest_field > max_largest_field {
            Some(Player::Min)
        } else if max_largest_field > min_largest_field {
            Some(Player::Max)
        } else {
            None
        };
    }
}
