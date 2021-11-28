# User guide

## How to install

Clone the git repository.

The program is written in Rust. To run it, you need Cargo, the Rust build tool. Cargo can be installed from https://www.rust-lang.org/tools/install.

## How to run

To run the program, run `cargo run --release` in the project folder. Note: It is important to use the `--release` flag which makes the program run about 100x faster. With the flag a turn will take only a few seconds on a modern computer, but without it it will take minutes.

## How to use

### Entering a board

The program will ask you to enter a board. A board is multiple lines of numbers that you need to paste into the terminal.

Example board:

```
       0   0   0   0
     0   0   0   0   0   0
   0   0   0   0   0  -16
 0  +16  0   0       0   0
       0   0   0   0   0   0
     0   0   0   0
```

The spaces in the beginning of lines are important as they affect the shape of the board.

After entering a board, write an empty line to continue. The program should now print the same board you just entered with colors.

### Interpreting the board

Battle sheep is played on a hexagonal grid and that is what the grid of numbers is trying to represent. Straight lines are found on the horizontal, descending diagonal and ascending diagonal axes. Note that the vertical axis does not represent a straight line even though it may look like so in some terminals.

There are two AI players in the game: Min and Max. Red tiles starting with - are Min's stacks and blue ones with + are Max's stacks. The number represents the stack size. Tiles with a green 0 are empty tiles.

Min always starts the game. The players take turns playing the game until one wins.

## How to test

Run `cargo test` in the project folder.
