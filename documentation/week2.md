This week there were a lot of student events so I had relatively little time to advance the project.

## What did I do?
- I implemented board parsing and writing.
- I created two tests for the parsing and writing.
- I researched how to do code coverage reporting in Rust.

## Questions
- Which code coverage tool to use?
- Should I generate code coverage report locally and commit it, or in Github?
- Should I put tests in the same file as the code (Rust unit tests convention) or a separate tests folder (Rust integration tests convention)?
- Would it optimize minimax if I ordered the next moves by their heuristic value, so the alpha-beta pruning would kick in sooner?

## What next?
- Add code coverage reporting.
- Design an algorithm for generating all possible (or sensible) next moves for a player.
- Design a simple heuristic for evaluating the game state. A more advanced heuristic can be done later.
- Implement the minimax tree algorithm.
- Correct the picture of the square grid representation in the specification document. It needs to be rotated 90°.
- Write myself a list of all the steps that must be done to complete the project.

## Time spent
12h