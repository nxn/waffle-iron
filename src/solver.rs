use crate :: sudoku :: { Sudoku, traits :: SudokuState };
use self :: { output :: * };

pub struct Solver {
    solution_limit: usize
}

impl Default for Solver { 
    fn default() -> Self  {
        Self { solution_limit: 1 }
    }
}

impl Solver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(solution_limit: usize) -> Self {
        Self { solution_limit }
    }

    pub fn solve(&self, puzzle: &[u8; 81]) -> SolverOutput {
        self.solve_state(&Sudoku::new(puzzle))
    }

    pub fn solve_state<T: SudokuState>(&self, state: &T) -> SolverOutput {
        let mut output = SolverOutput { 
            steps: state.remaining(),
            iterations: 0,
            result: Vec::default()
        };

        self.solve_rec(state, 0, &mut output);
        
        output
    }

    fn solve_rec<T: SudokuState>(&self, state: &T, bf: usize, output: &mut SolverOutput) {
        if state.remaining() == 0 {
            let mut solution = [0u8; 81];
            for (i, &value) in state.iter().enumerate() {
                solution[i] = value;
            }
    
            let rec = SolutionRecord {
                solution,
                iteration: output.iterations,
                branches: bf,
            };
    
            output.result.push(rec);
        }
    
        if self.solution_limit > 0 && output.result.len() >= self.solution_limit {
            return;
        }
    
        output.iterations += 1;
    
        let choices = state.next_choice();
        let bc = choices.len();
    
        for (index, value) in choices {
            self.solve_rec(&state.set(index, value), bf + (bc - 1).pow(2), output);
        }
    }
}

pub mod output {
    use crate :: format :: { BoxFormat, Alignment };

    pub struct SolverOutput {
        pub steps: usize,
        pub iterations: usize,
        pub result: Vec<SolutionRecord>,
    }

    pub struct SolutionRecord {
        pub iteration: usize,
        pub branches: usize,
        pub solution: [u8; 81],
    }

    impl std::fmt::Display for SolverOutput {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut boxf = BoxFormat::new(formatter);

            boxf.header("Waffle-Iron", None)?;
            for solution in self.result.iter() {
                boxf.line_break()?
                    .content(&format!("{}", solution), None)?;
            }
            boxf.footer("", None)?;

            Ok(())
        }
    }

    impl std::fmt::Debug for SolverOutput {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let summary = format!(
                "Iterations: {}, Steps: {}", self.iterations, self.steps
            );

            let mut boxf = BoxFormat::new(formatter);
            boxf.header("Waffle-Iron", None)?;
            
            for result in self.result.iter() {
                let summary = format!("Iteration: {}, Branches: {}", result.iteration, result.branches);
                boxf.section(&summary, Alignment::Center.into())?
                    .empty_line()?
                    .content(&format!("{}", result), None)?
                    .empty_line()?;
            }

            boxf.section("Summary", None)?
                .content(&summary, Alignment::Center.into())?
                .footer("", None)?;

            Ok(())
        }
    }

    impl std::fmt::Display for SolutionRecord {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for num in self.solution.iter() {
                write!(formatter, "{}", num)?;
            }
            Ok(())
        }
    }

    impl std::fmt::Debug for SolutionRecord {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let summary = format!("Iteration: {}, Branches: {}", self.iteration, self.branches);

            BoxFormat::new(formatter)
                .header("Waffle-Iron", None)?
                .empty_line()?
                .content(&format!("{}", self), None)?
                .empty_line()?
                .section("Summary", None)?
                .content(&summary, Alignment::Center.into())?
                .footer("", None)?;
            
            Ok(())
        }
    }
}