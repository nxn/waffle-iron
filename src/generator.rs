use crate::{
    sudoku  :: { Sudoku, traits  :: SudokuState },
    solver  :: { Solver },
    random  :: { traits :: Random }
};

use self :: { output::* };

use std::collections::HashSet;

lazy_static! {
    static ref INDICES: HashSet<usize>  = (0..81).collect();
    static ref DIGITS:  HashSet<u8>     = (1..10).collect();
}

pub struct Generator {
    samples:            u8,
    sample_iterations:  u8,
    iteration_removals: u8
}

impl Default for Generator {
    fn default() -> Self {
        Self {
            samples:            21,
            sample_iterations:  58,
            iteration_removals: 1
        }
    }
}

impl Generator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn samples(&mut self, samples: u8) -> &mut Self {
        self.samples = samples;

        self
    }

    pub fn iterations(&mut self, sample_iterations: u8) -> &mut Self {
        // The lowest/minimum known amount of values that can compose a uniquely solvable sudoku board is 17.
        // Therefore there is no point in iterating over a puzzle/sample more than 64 times as it will essentially
        // be unsolvable beyond this point.
        self.sample_iterations = std::cmp::min(sample_iterations, 64);

        // Make sure removal per iteration obeys cap based on the new sample_iterations value
        self.removals(self.iteration_removals);

        self
    }

    pub fn removals(&mut self, iteration_removals: u8) -> &mut Self {
        // Caps removals per iteration based on the amount of iterations being performed. As mentioned above, there 
        // is little point in performing more than 64 total removals per sample as the puzzle will no longer be
        // solvable after this point.
        self.iteration_removals = std::cmp::min(
            std::cmp::max(iteration_removals, 1), // Minimum amount of removals must be at least 1
            64 / self.sample_iterations
        );

        self
    }

    pub fn generate(&self) -> GeneratorOutput {
        let solver = Solver::with_limit(2);
        let solve_output = Solver::with_limit(1).solve(&prefill_solution());
        let starting_state = Sudoku::new(&solve_output.result[0].solution);
        let mut random_index = INDICES.iter().random();

        let mut best_state = starting_state.clone();
        let mut best_difficulty = 0;
        let mut best_solve_output = solve_output;
    
        // Somewhat counter intuitively, rather than continually iterating on the best puzzle found so far, starting
        // each sample directly from the solution often leads to better results. After a certain point the amount of
        // removals we can perform while still maintaining a solvable puzzle shrinks drastically and most iterations end
        // up having to be undone. When starting each sample from the unaltered solution we have a better chance of 
        // randomly stumbling into a good series of removals early -- or so it seems.
        for _ in 0 .. self.samples {
            let mut sample_state = starting_state.clone();
            let mut sample_difficulty = 0;

            for _ in 0 .. self.sample_iterations {
                let mut state = sample_state.set(*random_index.next().unwrap(), 0);
    
                for _ in 0 .. self.iteration_removals - 1 {
                    state = state.set(*random_index.next().unwrap(), 0);
                }
    
                let output = solver.solve_state(&state);
                
                if output.result.len() > 1 {
                    continue;
                }
        
                let difficulty = output.result[0].branches * 100 + state.remaining();
                if sample_difficulty < difficulty {
                    sample_state        = state;
                    sample_difficulty   = difficulty
                }
            }
    
            if best_difficulty < sample_difficulty  {
                best_state      = sample_state;
                best_difficulty = sample_difficulty;
            }

            random_index.reset();
        }
    
        // Complete removing values from the best sample found
        for &index in random_index {
            if let Some(value) = best_state.get(index) {
                if value == 0 { continue; }
            }
    
            let new_state = best_state.set(index, 0);
            let output = solver.solve_state(&new_state);
            
            if output.result.len() == 1 {
                best_state = new_state;
                best_solve_output = output;
            }
        }
    
        GeneratorOutput {
            difficulty: best_solve_output.result[0].branches * 100 + best_state.remaining(),
            solution:   best_solve_output.result[0].solution,
            puzzle:     best_state.into(),
        }
    }
}

fn prefill_solution() -> [u8; 81] {
    let mut grid = vec![];
    (0..9).for_each(|_| grid.push(vec![]));

    fill_box_one(&mut grid);
    fill_box_two(&mut grid);
    fill_box_three(&mut grid);
    fill_remaining(&mut grid);
    
    let mut output = [0; 81];
    for (i, &item) in grid.iter().flatten().enumerate() {
        output[i] = item;
    }
    output
}

fn fill_box_one(grid: &mut Vec<Vec<u8>>) {
    let mut choices = DIGITS.iter().random();
    for row in grid.iter_mut().take(3) {
        (0..3).for_each(|_| row.push(*choices.next().unwrap()))
    }
}

fn fill_box_two(grid: &mut Vec<Vec<u8>>) {
    let box1_row1: HashSet<u8> = grid[0].iter().take(3).copied().collect();
    let box1_row2: HashSet<u8> = grid[1].iter().take(3).copied().collect();
    let box1_row3: HashSet<u8> = grid[2].iter().take(3).copied().collect();
    let mut used = HashSet::new();

    // Populate first row of second box (must exclude first three items from first box)
    let mut choices = DIGITS.difference(&box1_row1).random();
    for _ in 0..3 {
        let choice = *choices.next().unwrap();
        grid[0].push(choice);
        used.insert(choice);
    }

    // Check which of the remaining numbers exist in the 2nd and 3rd rows of the first box. These must be prioritized 
    // and placed into the two remaining rows of this box. If the selected number was not found, skip it for now.
    let used_copy = used.iter().copied().collect();
    for &choice in DIGITS.difference(&used_copy).random() {
        if box1_row2.contains(&choice) {
            grid[2].push(choice);
            used.insert(choice);
        }
        else if box1_row3.contains(&choice) {
            grid[1].push(choice);
            used.insert(choice);
        }
    }

    // Fill the rest of the second box with the remaining choices.
    for &choice in DIGITS.difference(&used).random() {
        if grid[1].len() < grid[2].len() {
            grid[1].push(choice);
        }
        else {
            grid[2].push(choice);
        }
    }
}

fn fill_box_three(grid: &mut Vec<Vec<u8>>) {
    for row in grid.iter_mut().take(3) {
        row.extend(
            DIGITS.difference(&row.iter().take(6).copied().collect()).random()
        );
    }
}

// Sets first row value to a random number and fills the rest of the row with 0s
fn fill_remaining(grid: &mut Vec<Vec<u8>>) {
    let box1_col1 = grid.iter().take(3).map(|r| r[0]).collect();
    let mut choices = DIGITS.difference(&box1_col1).random();

    for row in grid[3..9].iter_mut() {
        row.push(*choices.next().unwrap());
        (1..9).for_each(|_| row.push(0));
    }
}

pub mod output {
    use std::fmt::Write;
    use crate :: format  :: { BoxFormat, Alignment };

    pub struct GeneratorOutput {
        pub puzzle: [u8; 81],
        pub solution: [u8; 81],
        pub difficulty: usize,
    }

    impl std::fmt::Display for GeneratorOutput {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut puzzle = String::new(); 
            for val in self.puzzle.iter() {
                write!(&mut puzzle, "{}", val)?;
            }

            BoxFormat::new(formatter)
                .header("Waffle-Iron", None)?
                .line_break()?
                .content(&puzzle, None)?
                .footer("", None)?;

            Ok(())
        }
    }

    impl std::fmt::Debug for GeneratorOutput {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut puzzle = String::new(); 
            for val in self.puzzle.iter() {
                write!(&mut puzzle, "{}", val)?;
            }

            let mut solution = String::new();
            for val in self.solution.iter() {
                write!(&mut solution, "{}", val)?;
            }

            let summary = format!("Difficulty: {}", self.difficulty);

            BoxFormat::new(formatter)
                .header("Waffle-Iron", None)?
                .empty_line()?
                .content(&puzzle, None)?
                .empty_line()?
                .section("Solution", None)?
                .empty_line()?
                .content(&solution, None)?
                .empty_line()?
                .section("Summary", None)?
                .content(&summary, Alignment::Center.into())?
                .footer("", None)?;

            Ok(())
        }
    }
}