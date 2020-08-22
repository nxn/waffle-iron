# waffle-iron
Rust sudoku puzzle generator and solver that can compile to WASM. Loosely based on [Daniel Beer's approach](https://dlbeer.co.nz/articles/sudoku.html) to solving and generating sudoku 
puzzles.

Build with:

    cargo build --release

To generate wasm use wasm-pack:

    wasm-pack build --release

To run, execute the 'wi-exec' binary, or use:

    cargo run --release -- *any arguments you want to pass in go here*

Running the executable without any inputs will generate a sudoku puzzle string. If you want to automatically see the puzzle's solution you can do so by passing in the --verbose flag.

To solve a puzzle pass in an 81 digit long number (0s indicate empty cells) with **-p** or **--puzzle=**"number".

### Arguments:

Argument&nbsp;Name     |  Shorthand            | Description
-----------------------|-----------------------|------------
**--verbose**          | **-v**                | Prints additional info and statistics; will print solution when used to generate puzzles.
**--limit=**[0-9]      | **-l**[0-9]           | Limits the number of solutions the solver will attempt to find. Useful when confirming a puzzle only has a single solution. Default behavior is to end solving after finding the first solution.
**--puzzle=**[0-9]{81} | **-p**&nbsp;[0-9]{81} | Solves the passed in puzzle string

### Examples:

Generate new puzzle and its solution:

    wi-exec -v

Solve a puzzle:

    wi-exec -v -p 000230070050000000400000850230900500060004093008000000040007030800000000020009061

See whether a puzzle has more than one solution:

    wi-exec -v -l2 -p 295743861431865900876192543387459216612387495549216738763534189928671354154938600

