use either::Either;
use next_gen::prelude::*;
use std::{
    error::Error,
    iter,
    ops::{Index, IndexMut},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Player(pub u8);

impl Player {
    pub const PLAYER_COUNT: usize = 2;

    pub fn iter() -> impl Iterator<Item = Player> {
        return (0..Self::PLAYER_COUNT as u8).map(|id| Player(id));
    }

    pub const fn id(self) -> usize {
        return self.0 as usize;
    }

    /* The direction where this player is trying to push the game value. */
    pub fn direction(self) -> i32 {
        return match self.0 {
            0 => -1,
            1 => 1,
            _ => unreachable!(),
        };
    }

    /* The player whose turn is next. */
    pub fn next(self) -> Player {
        return Player((self.0 + 1) % Self::PLAYER_COUNT as u8);
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
 * 1 bits player
 * 5 bits stack_size, offset by -1
 * Numerically:
 * 0-31 = Player 0 Stack with size 1-32
 * 32-63 = Player 1 Stack with size 1-32
 * 64-127 = NoTile
 * 128-255 = Empty */
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Tile(pub u8);

impl Tile {
    pub const MAX_STACK_SIZE: u8 = 32;

    pub const NO_TILE: Self = Self::new(TileType::NoTile, Player(0), 1);
    pub const EMPTY: Self = Self::new(TileType::Empty, Player(0), 1);

    pub const fn new(tile_type: TileType, player: Player, stack_size: u8) -> Self {
        let bitfield = stack_size - 1
            + player.0 * 32
            + match tile_type {
                TileType::Stack => 0,
                TileType::NoTile => 64,
                TileType::Empty => 128,
            };
        return Self(bitfield);
    }

    pub fn stack(player: Player, stack_size: u8) -> Self {
        return Self::new(TileType::Stack, player, stack_size);
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
        return Player(self.0 / 32);
    }

    pub fn stack_size(self) -> u8 {
        return self.0 % 32 + 1;
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
pub const DIRECTION_OFFSETS: [(isize, isize); 6] =
    [(0, 1), (1, 1), (1, 0), (0, -1), (-1, -1), (-1, 0)];

pub fn add_offset((r, q): (isize, isize), (off_r, off_q): (isize, isize)) -> (isize, isize) {
    return (r + off_r, q + off_q);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Board {
    /* Tiles stored in row-major order. */
    pub tiles: Vec<Tile>,
    pub row_length: usize,
}

impl Index<(isize, isize)> for Board {
    type Output = Tile;

    fn index(&self, coords: (isize, isize)) -> &Self::Output {
        /* Return the tile for all valid coords in the board, but also return NoTile for all coords
         * outside the board. This way the indexing operation never panics. */
        if self.coords_in_range(coords) {
            return &self.tiles[self.coords_to_index(coords)];
        } else {
            return &Tile::NO_TILE;
        }
    }
}

impl IndexMut<(isize, isize)> for Board {
    fn index_mut(&mut self, coords: (isize, isize)) -> &mut Self::Output {
        let index = self.coords_to_index(coords);
        return &mut self.tiles[index];
    }
}

impl Board {
    pub fn num_rows(&self) -> usize {
        return self.tiles.len() / self.row_length;
    }

    pub fn coords_to_index(&self, (r, q): (isize, isize)) -> usize {
        /* r = row coordinate, q = column coordinate */
        let (r, q) = (r as usize, q as usize);
        return self.row_length * r + q;
    }

    pub fn index_to_coords(&self, index: usize) -> (isize, isize) {
        return (
            (index / self.row_length) as isize,
            (index % self.row_length) as isize,
        );
    }

    pub fn coords_in_range(&self, (r, q): (isize, isize)) -> bool {
        let (r, q) = (r as usize, q as usize);
        return r < self.num_rows() && q < self.row_length;
    }

    /* Iterates through all tiles in row-major order. */
    pub fn iter_row_major(&self) -> impl Iterator<Item = ((isize, isize), Tile)> + '_ {
        return self
            .tiles
            .iter()
            .enumerate()
            .map(|(index, &tile)| (self.index_to_coords(index), tile));
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = (usize, &[Tile])> {
        return self.tiles.chunks_exact(self.row_length).enumerate();
    }

    /* Iterates through all neighbors of the given coordinates in clockwise direction. */
    pub fn iter_neighbors(
        &self,
        coords: (isize, isize),
    ) -> impl Iterator<Item = ((isize, isize), Tile)> + '_ {
        return DIRECTION_OFFSETS.iter().map(move |&offset| {
            let neighbor_coords = add_offset(coords, offset);
            (neighbor_coords, self[neighbor_coords])
        });
    }

    pub fn iter_empty_straight_line(
        &self,
        start_coords: (isize, isize),
        direction: (isize, isize),
    ) -> impl Iterator<Item = (isize, isize)> + '_ {
        return iter::successors(Some(start_coords), move |&coords| {
            let next_coords = add_offset(coords, direction);
            if self[next_coords].is_empty() {
                Some(next_coords)
            } else {
                None
            }
        })
        .skip(1);
    }

    pub fn iter_empty_straight_line_ends(
        &self,
        start_coords: (isize, isize),
    ) -> impl Iterator<Item = (isize, isize)> + '_ {
        return DIRECTION_OFFSETS.iter().filter_map(move |&direction| {
            self.iter_empty_straight_line(start_coords, direction)
                .last()
        });
    }

    pub fn iter_empty_outer_edge(&self) -> impl Iterator<Item = (isize, isize)> + '_ {
        #[generator((isize, isize))]
        fn generate_edge(board: &Board) {
            /* We know that the first board tile we encounter must be on the outer edge. */
            let (start_coords, start) = board
                .iter_row_major()
                .find(|&(_, tile)| tile.is_board_tile())
                .expect("The board is empty");

            /* The first board tile we encountered must be on the left edge of the board, so its
             * left side (offset (0, -1)) is a safe direction to start iterating neighbors. */
            let mut previous_coords = add_offset(start_coords, (0, -1));
            let mut coords = start_coords;

            /* Iterate along the outer edge of the board. */
            loop {
                /* Search through the neighbors of coords in clockwise direction starting from
                 * previous_coords. Find the first board tile. That board tile must also be on
                 * the outer edge. */
                let (next_coords, next) = board
                    .iter_neighbors(coords)
                    .chain(board.iter_neighbors(coords))
                    .skip_while(|&(neighbor_coords, _)| neighbor_coords != previous_coords)
                    .skip(1)
                    .find(|&(_, neighbor)| neighbor.is_board_tile())
                    .unwrap_or((start_coords, start));

                if next.is_empty() {
                    yield_!(next_coords);
                }

                /* We have come a full circle. */
                if next_coords == start_coords {
                    break;
                }

                previous_coords = coords;
                coords = next_coords;
            }
        }

        mk_gen!(let generator = box generate_edge(self));
        return generator.into_iter();
    }

    /* Extends the board by one in any direction. */
    pub fn extend_to_contain(&mut self, (r, q): (isize, isize)) -> (isize, isize) {
        let (mut offset_r, mut offset_q) = (0, 0);

        if r == self.num_rows() as isize {
            /* Add a new row after. */
            self.tiles
                .extend(iter::repeat(Tile::NO_TILE).take(self.row_length));
        } else if r == -1 {
            /* Add a new row before. */
            self.tiles
                .splice(0..0, iter::repeat(Tile::NO_TILE).take(self.row_length));

            /* Rows have shifted forward by one. */
            offset_r = 1;
        }

        if q == self.row_length as isize {
            /* Add a new column after. */
            let num_rows = self.num_rows();
            self.row_length += 1;

            /* Inserting a new tile to the end of every row. */
            for i in 0..num_rows {
                self.tiles
                    .insert(i * self.row_length + (self.row_length - 1), Tile::NO_TILE)
            }
        } else if q == -1 {
            /* Add a new column before. */
            let num_rows = self.num_rows();
            self.row_length += 1;

            /* Inserting a new tile to the beginning of every row. */
            for i in 0..num_rows {
                self.tiles.insert(i * self.row_length, Tile::NO_TILE)
            }

            /* Columns have shifted forward by one. */
            offset_q = 1;
        }

        return (offset_r, offset_q);
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
                } else {
                    let player = match &tile_content[..1] {
                        "-" => Player(0),
                        "+" => Player(1),
                        _ => return Err("Invalid tile")?,
                    };

                    let stack_size = tile_content[1..].parse::<u8>()?;
                    if stack_size > Tile::MAX_STACK_SIZE {
                        return Err(format!("Stack size over {}", Tile::MAX_STACK_SIZE))?;
                    } else if stack_size == 0 {
                        return Err("Stack size is 0")?;
                    }

                    tiles.push(Tile::stack(player, stack_size));
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

            for &tile in row.iter() {
                let tile_string = match tile.tile_type() {
                    TileType::NoTile => format!("    "),
                    TileType::Empty => {
                        if colored {
                            format!("{} 0  {}", GREEN, RESET)
                        } else {
                            format!(" 0  ")
                        }
                    }
                    TileType::Stack => {
                        let (symbol, color) = match tile.player() {
                            Player(0) => ("-", RED),
                            Player(1) => ("+", BLUE),
                            _ => unreachable!(),
                        };
                        if colored {
                            format!("{}{}{:<3}{}", color, symbol, tile.stack_size(), RESET)
                        } else {
                            format!("{}{:<3}", symbol, tile.stack_size())
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
        return self
            .iter_row_major()
            /* Check if the tile is a splittable stack of this player. */
            .filter(move |(_, tile)| {
                tile.is_stack() && tile.player() == player && tile.stack_size() > 1
            })
            .flat_map(move |(origin_coords, stack)| {
                self.iter_empty_straight_line_ends(origin_coords)
                    .flat_map(move |target_coords| {
                        /* Iterate through all the ways to split the stack. */
                        (1..stack.stack_size()).map(move |split| {
                            let mut next_board = self.clone();
                            next_board[target_coords] = Tile::stack(player, split);
                            next_board[origin_coords] =
                                Tile::stack(player, stack.stack_size() - split);

                            next_board
                        })
                    })
            });
    }

    /* Iterates through starting moves where player places a stack on the outer edge. */
    fn possible_starting_moves(&self, player: Player) -> impl Iterator<Item = Board> + '_ {
        return self.iter_empty_outer_edge().map(move |coords| {
            let mut next_board = self.clone();
            next_board[coords] = Tile::stack(player, 16);

            next_board
        });
    }

    /* Evaluates the current board state. The more the value is in one player's direction, the more
     * advantage they have. This is a very simple evaluation function that checks how blocked the
     * stacks are by their neighbors and how evenly split they are. In the endgame, another
     * heuristic is used. */
    pub fn heuristic_evaluate(&self) -> i32 {
        let mut value = 0;
        let mut player_all_blocked = [true; Player::PLAYER_COUNT];
        let mut player_stacks = [0; Player::PLAYER_COUNT];

        let mut player_smallest_stack = [u8::MAX; Player::PLAYER_COUNT];
        let mut player_largest_stack = [0; Player::PLAYER_COUNT];

        for (coords, tile) in self.iter_row_major() {
            if tile.is_stack() {
                let player = tile.player();
                let size = tile.stack_size();

                player_stacks[player.id()] += 1;
                player_largest_stack[player.id()] =
                    u8::max(player_largest_stack[player.id()], size);
                player_smallest_stack[player.id()] =
                    u8::min(player_smallest_stack[player.id()], size);

                /* A maximum of 6 directions are blocked. */
                let mut blocked_directions = 6;
                for (_, neighbor) in self.iter_neighbors(coords) {
                    if neighbor.is_empty() {
                        blocked_directions -= 1;
                    }
                }

                if size > 1 && blocked_directions < 6 {
                    player_all_blocked[player.id()] = false;
                }

                /* Being surrounded from more sides and having more sheep in the stack increase
                 * its blocked score. */
                let blocked_score = (size as i32 - 1) * blocked_directions;

                /* A blocked stack gives a disadvantage to the player, so the board value is moved
                 * away from the player's direction. */
                value -= blocked_score * player.direction();
            }
        }

        /* Extra score for splitting stacks evenly. This does not matter as much as being blocked,
         * the maximum splitting bonus is 7. */
        for player in Player::iter() {
            let uneven_score = (player_largest_stack[player.id()] as i32
                - player_smallest_stack[player.id()] as i32)
                / 2;
            value -= uneven_score * player.direction();
        }

        /* If all players are blocked, the game is over and the winner can be determined. */
        if player_all_blocked.iter().all(|&b| b) {
            /* All players who have the most stacks. */
            let most_stacks = *player_stacks.iter().max().unwrap();
            let most_stack_holders = Player::iter()
                .filter(|p| player_stacks[p.id()] == most_stacks)
                .collect::<Vec<_>>();

            let largest_fields = self.largest_connected_fields();

            /* All players who have the largest fields out of those who have the most stacks. */
            let largest_field = most_stack_holders
                .iter()
                .map(|p| largest_fields[p.id()])
                .max()
                .unwrap();
            let winners = most_stack_holders
                .iter()
                .filter(|p| largest_fields[p.id()] == largest_field);

            /* Set value to one million in the winners' directions. */
            value = 0;
            for &player in winners {
                value += 1000000 * player.direction();
            }
        }

        return value;
    }

    /* Returns the largest connected fields for every player. */
    pub fn largest_connected_fields(&self) -> [u32; Player::PLAYER_COUNT] {
        let mut player_largest_field = [0; Player::PLAYER_COUNT];

        let mut visited = vec![false; self.tiles.len()];
        let mut dfs_stack = Vec::<(isize, isize)>::new();

        for (start_coords, tile) in self.iter_row_major() {
            if tile.is_stack() && !visited[self.coords_to_index(start_coords)] {
                let player = tile.player();
                let mut field_size = 0;

                /* Depth-first search for counting the size of a connected field. */
                visited[self.coords_to_index(start_coords)] = true;
                dfs_stack.push(start_coords);
                while let Some(coords) = dfs_stack.pop() {
                    field_size += 1;

                    for (neighbor_coords, neighbor) in self.iter_neighbors(coords) {
                        if neighbor.is_stack()
                            && neighbor.player() == player
                            && !visited[self.coords_to_index(neighbor_coords)]
                        {
                            visited[self.coords_to_index(neighbor_coords)] = true;
                            dfs_stack.push(neighbor_coords);
                        }
                    }
                }

                player_largest_field[player.id()] =
                    u32::max(player_largest_field[player.id()], field_size);
            }
        }

        return player_largest_field;
    }
}
