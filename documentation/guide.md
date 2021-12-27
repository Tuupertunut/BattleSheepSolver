# User guide

## How to install

Clone the git repository.

The program is written in Rust. To run it, you need Cargo, the Rust build tool. Cargo can be installed from https://www.rust-lang.org/tools/install.

## How to run

There are two game modes in this program: watch mode and play mode. Game mode is selected with a command line flag `-w` or `-p`.

To run the program in watch mode, run `cargo run --release -- -w` in the project folder.

To run it in play mode, run `cargo run --release -- -p`.

Note: It is important to use the `--release` flag which makes the program run about 100x faster.

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

There are two players in the game: Min and Max. Red tiles starting with - are Min's stacks and blue ones with + are Max's stacks. The number represents the stack size. Tiles with a green 0 are empty tiles.

### Game modes

#### Watch mode

In watch mode, you watch two AIs play against each other. You enter a starting board, and the AIs will play until the game ends. Min always starts the game.

#### Play mode

In play mode, you play against the AI. On each turn, you enter a new board where you have made your move. The AI will then respond by making its move and printing out the board.

You play as Max and your AI opponent is Min. You can choose who starts by either entering a fresh starting board or one where you have made the first move.

A good way to play is to copy the AI's output board into a text editor, make your move, and paste it back to the terminal.

## How to test

Run `cargo test` in the project folder.
