I was ill most of the week and there was nothing else to do than advance this project.

## What did I do?

-   I added code coverage reporting. It uses cargo-tarpaulin to generate a report in Github Actions and upload it to codecov.io.
-   I designed and implemented an algorithm for generating next moves for a player.
-   I designed and implemented a heuristic for evaluating the game state. It is based on how many neighbors a stack has. It is very crude but it correlates with having an advantage in the game.
-   I refactored and moved all board functions into a separate board type.
-   I did a lot of research on how to optimize the minimax algorithm in the future.
-   I came up with a better way to represent the hex grid in command line.

## Questions

Currently none.

## What next?

-   Implement the minimax tree algorithm.
-   Optimize the algorithm and the program in general with many tricks I have thought of.
-   Implement the better hex grid UI in command line and edit the specification document.
-   Refactor more of the board code.

## Time spent

42h
