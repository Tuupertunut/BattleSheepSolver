use std::error::Error;

#[derive(Debug)]
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
    println!("{}", write_board(&board));
}

fn parse_board(input: &str) -> Result<Vec<Vec<Tile>>, Box<dyn Error>> {
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
                row.push(Tile::MaxStack(stack_size));
            } else if tile_content.starts_with("-") {
                let stack_size = tile_content[1..].parse::<u8>()?;
                row.push(Tile::MinStack(stack_size));
            } else {
                return Err("Invalid tile")?;
            }
        }

        board.push(row);
    }

    return Ok(board);
}

fn write_board(board: &Vec<Vec<Tile>>) -> String {
    let mut output = String::new();

    for row in board {
        for tile in row {
            let tile_string = match tile {
                &Tile::NoTile => format!("    "),
                &Tile::Empty => format!(" 0  "),
                &Tile::MaxStack(stack_size) => format!("+{:<3}", stack_size),
                &Tile::MinStack(stack_size) => format!("-{:<3}", stack_size),
            };
            output.push_str(&tile_string);
        }
        output.push_str("\n")
    }

    return output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_equals_input() {
        let input = " 0  +2       0  \
                   \n-1   0  -3  +2  \
                   \n     0  \
                   \n";
        assert_eq!(input, write_board(&parse_board(input).unwrap()));
    }

    #[test]
    fn parse_fails_on_invalid_board() {
        assert!(parse_board("abcdefg").is_err());
    }
}
