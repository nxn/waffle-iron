#[macro_use] extern crate lazy_static;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate arr_macro;

mod sudoku;

mod candidates;
mod values;
mod solver;
mod generator;

mod bitsets;
mod random;
mod indices;
mod format;

pub use {
    sudoku      :: { Sudoku },
    solver      :: { Solver },
    generator   :: { Generator },
};

pub mod traits {
    use super::sudoku;
    pub use { 
        sudoku :: traits :: SudokuState
    };
}

#[cfg(target_arch = "wasm32")]
mod wasm;