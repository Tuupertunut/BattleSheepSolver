use std::error::Error;

enum Tile {
    NoTile, /* outside of the board */
    Empty,
    MaxStack(u8),
    MinStack(u8),
}

fn main() {
    println!("Enter a board");
    let mut input_buffer = String::new();
    while !input_buffer.ends_with("\n\n") {
        std::io::stdin()
            .read_line(&mut input_buffer)
            .expect("Input contained illegal characters");
    }
    let board = parse_board(&input_buffer).expect("Input is not a valid board");
}

fn parse_board(input: &str) -> Result<Vec<Vec<Tile>>, Box<dyn Error>> {
    let mut rows = Vec::<Vec<Tile>>::new();

    for row_string in input.trim_end().split("\n") {
        let mut tiles = Vec::<Tile>::new();

        /* Splitting row into 4 character pieces. */
        for tile_string in row_string
            .trim_end()
            .as_bytes()
            .chunks(4)
            .map(String::from_utf8_lossy)
        {
            let tile_content = tile_string.trim();

            if tile_content == "" {
                tiles.push(Tile::NoTile);
            } else if tile_content == "0" {
                tiles.push(Tile::Empty);
            } else if tile_content.starts_with("+") {
                let stack_size = tile_content[1..].parse::<u8>()?;
                tiles.push(Tile::MaxStack(stack_size));
            } else if tile_content.starts_with("-") {
                let stack_size = tile_content[1..].parse::<u8>()?;
                tiles.push(Tile::MinStack(stack_size));
            } else {
                return Err("Invalid tile")?;
            }
        }

        rows.push(tiles);
    }

    return Ok(rows);
}
