# Testing report

## Unit testing

The minimax algorithm and the game board have many complex functions where it is not intuitively clear that they work correctly. For this reason they need to be properly unit tested.

Currently, the board code is very thoroughly tested. Most of the tests test the heuristic evaluation function, as it needs to satisfy certain conditions. For example, the heuristic needs to always evaluate a winning state higher than any non-winning state. The boards for testing the functions have been selected to contain as many corner cases as possible.

The minimax algorithm is still constantly changing and not yet tested.

### Test coverage

Up-to-date test coverage report can be found in [codecov.io](https://app.codecov.io/gh/Tuupertunut/BattleSheepSolver). It is automatically generated for each commit in Github Actions.

### Implementation

The unit tests are in the `src/tests.rs` file.

They can be run by executing the command `cargo test` in the project folder.

Code coverage report can be generated by installing [Tarpaulin](https://github.com/xd009642/tarpaulin) and running it. Github Actions also automatically generates a code coverage report using Tarpaulin and uploads it into codecov.io.

## Performance testing

Some performance testing has been done to compare different changes to the code during development.

The performance test only provides guidelines for how the performance of one test case has changed on one machine. The numbers may not be reproducable and they are not very accurate.

### Test setup

-   CPU: Intel Core i5-7200U
-   OS: Ubuntu 20.04
-   Rust compiler: multiple versions
-   Board: The example board in the [user guide](guide.md)
-   Command: `cargo run --release`

### Results

The numbers below tell how long it took to run the first turn of the game after a change in the code.

| Change                                                        | Time   |
| ------------------------------------------------------------- | ------ |
| Naive minimax algorithm, depth 4 turns, no optimizations      | 5 s    |
| Store board in a single long array instead of array of arrays | 2.8 s  |
| Implement alpha-beta pruning                                  | 80 ms  |
| Increase depth to 5 turns                                     | 1.3 s  |
| Sort moves by heuristic                                       | 550 ms |
| Update Rust compiler from 1.56.1 to 1.57                      | 450 ms |
| Do not sort leaves of the minimax tree                        | 270 ms |
| Increase depth to 6 turns                                     | 3.2 s  |
| Generate moves dynamically (on demand) with an iterator       | 1.2 s  |