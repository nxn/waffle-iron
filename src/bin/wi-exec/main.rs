#[macro_use] extern crate bitflags;
//#[macro_use] extern crate arr_macro;
mod util;
mod compare;

use waffle_iron :: { Solver, Generator };

use std :: {  env };

const PUZZLES: [&str; 9] = [
    "309000400200709000087000000750060230600904008028050041000000590000106007006000104",
    "295743861431865900876192543387459216612387495549216738763534189928671354154938600",
    "005300000800000020070010500400005300010070006003200080060500009004000030000009700",
    "003000000809460702200018600000006070008000400070800000002940005406032807000000200",
    "000500012000003070000907540803400090020000000009000103700040000140000030090600200",
    "534008010000002090000007604000500100100000003009001000305400000080200000060700382",
    "000065000510003000080000060000006040040309018000080200300001009000000800091700400",
    "090001007060030050070200001804000009500010830000005000000842000000000200006000003",
    "089005030005018900070400000950200600000030002000004003400102500002000000090850000",
];

bitflags! {
    pub struct Flags: u16 {
        const VERBOSE = 0x0001;
    }
}

pub struct Args {
    pub flags: Flags,
    pub limit: usize,
    pub puzzle: Option<[u8; 81]>
}

impl Args {
    fn new() -> Self {
        let mut flags = Flags::empty();
        let mut puzzle = None;
        let mut limit = 1;

        let args: Vec<String> = env::args().collect();
        for arg in args.into_iter() {
            if arg == "-v" || arg == "--verbose" {
                flags.insert(Flags::VERBOSE);
            }

            if arg.starts_with("-l") {
                if let Ok(num) = arg[2..].parse::<usize>() {
                    limit = num
                }
            }

            if arg.starts_with("--limit=") {
                if let Ok(num) = arg[8..].parse::<usize>() {
                    limit = num
                }
            }

            if arg.starts_with("-p") {
                if let Ok(num) = arg[2..].parse::<usize>() {
                    if num < PUZZLES.len() {
                        puzzle = util::puzzle_str_to_bytes(PUZZLES[num]);
                    }
                }
            }

            if arg.starts_with("--puzzle=") {
                if let Ok(num) = arg[9..].parse::<usize>() {
                    if num < PUZZLES.len() {
                        puzzle = util::puzzle_str_to_bytes(PUZZLES[num]);
                    }
                }
            }

            if arg.len() == 81 {
                puzzle = util::puzzle_str_to_bytes(&arg);
            }
        }

        Self { flags, puzzle, limit }
    }
}

// fn branch_distributions(limit: u32, config: GenerateOptions) {
//     println!("Solution Sample");
//     let mut map = std::collections::BTreeMap::<usize, usize>::default();
//     let now = std::time::Instant::now();
//     for _ in 0..limit {
//         let sudoku = Sudoku::generate(config.into());
//         let output = sudoku.solve(SolveOptions { solution_limit: 1 }.into());
//         let branches = output.result[0].branches;
//         if branches > 10 {
//             println!("{:?}", output);
//         }
//         let count = match map.get(&branches) {
//             Some(count) => *count,
//             None => 0
//         };
//         map.insert(branches, count + 1);
//     }
//     let dur = now.elapsed().as_millis();

//     for (key, value) in map.iter() {
//         println!("{},{}", key, value);
//     }

//     println!("Average time: {}ms", dur / u128::from(limit));
// }

// fn _rand_distributions(n: u8) {
//     println!("Solution Sample");
//     let mut map = std::collections::BTreeMap::<Vec<u8>, usize>::default();

//     let set: Vec<u8> = (1..n + 1).collect();
//     let random = &mut Random::from_iter(set.iter());

//     for _ in 0..600_000 {
//         *map.entry(random.copied().collect()).or_insert(0) += 1;
//     }

//     for (k,v) in map {
//         println!("{:?}: {}", k, v);
//     }
// }


fn main() {
    let args = Args::new();
    //let generate_options = GenerateOptions { samples: 21, sample_iterations: 58, iteration_removals: 1 };

    if let Some(puzzle) = args.puzzle {
        let time = std::time::Instant::now();
        let output = Solver::with_limit(args.limit).solve(&puzzle);
        let dur = time.elapsed().as_millis();

        println!();
        if args.flags.contains(Flags::VERBOSE) {
            print!("{:?}", output);
        }
        else {
            print!("{}", output);
        }
        println!();
        println!("Time: {}ms", dur);
    }
    else if true {
        let time = std::time::Instant::now();
        let output = Generator::new().generate();
        let dur = time.elapsed().as_millis();
    
        println!();
        if args.flags.contains(Flags::VERBOSE) {
            print!("{:?}", output);
        }
        else {
            print!("{}", output);
        }
        println!();
        println!("Time: {}ms", dur);
    }

    // let nums: Vec<usize> = (0..512).collect();
    // let rand = Random::new(nums.iter());

    // for r in rand {
    //     //print!("{}", r);
    // }
    //_rand_distributions(4);
}

