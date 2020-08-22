use crate :: { Generator, Solver };

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn generate(config: Option<GeneratorConfig>) -> GeneratorOutput {
    let mut gen = Generator::new();

    if let Some(options) = config {
        gen .samples(options.samples)
            .iterations(options.iterations)
            .removals(options.removals);
    }

    GeneratorOutput(gen.generate())
}

#[wasm_bindgen]
pub fn solve(puzzle: &[u8], config: Option<SolverConfig>) -> Result<SolverOutput, JsValue> {
    if puzzle.len() != 81 {
        return Err(JsValue::from_str("Invalid input"));
    }

    let solver = match config {
        Some(options) => Solver::with_limit(options.limit),
        None => Solver::new()
    };

    let mut input = [0; 81];
    for (i, &cell_value) in puzzle.iter().enumerate() {
        input[i] = cell_value;
    }
    
    Ok(SolverOutput(solver.solve(&input)))
}

#[wasm_bindgen]
pub struct SolverConfig { 
    pub limit: usize 
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self { limit: 1 }
    }
}

#[wasm_bindgen]
impl SolverConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
pub struct GeneratorConfig {
    pub samples:    u8,
    pub iterations: u8,
    pub removals:   u8
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            samples:    10,
            iterations: 29,
            removals:   2
        }
    }
}

#[wasm_bindgen]
impl GeneratorConfig {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
pub struct GeneratorOutput(crate::generator::output::GeneratorOutput);

#[wasm_bindgen]
impl GeneratorOutput {
    #[wasm_bindgen(getter)]
    pub fn puzzle(&self) -> Box<[u8]> {
        Box::from(self.0.puzzle)
    }

    #[wasm_bindgen(getter)]
    pub fn solution(&self) -> Box<[u8]> {
        Box::from(self.0.solution)
    }

    #[wasm_bindgen(getter)]
    pub fn difficulty(&self) -> usize {
        self.0.difficulty
    }
}


#[wasm_bindgen]
pub struct SolutionRecord(crate::solver::output::SolutionRecord);

#[wasm_bindgen]
impl SolutionRecord {
    #[wasm_bindgen(getter)]
    pub fn iteration(&self) -> usize {
        self.0.iteration
    }

    #[wasm_bindgen(getter)]
    pub fn branches(&self) -> usize {
        self.0.branches
    }

    #[wasm_bindgen(getter)]
    pub fn solution(&self) -> Box<[u8]> {
        Box::from(self.0.solution)
    }
}

#[wasm_bindgen]
pub struct SolverOutput(crate::solver::output::SolverOutput);

#[wasm_bindgen]
impl SolverOutput {
    #[wasm_bindgen(getter)]
    pub fn steps(&self) -> usize {
        self.0.steps
    }

    #[wasm_bindgen(getter)]
    pub fn iterations(&self) -> usize {
        self.0.iterations
    }

    #[wasm_bindgen(getter)]
    pub fn result(&mut self) -> Vec<JsValue>{
        let mut output = vec![];

        while let Some(rec) = self.0.result.pop() {
            output.push(JsValue::from(SolutionRecord(rec)));
        }

        output
    }
}
