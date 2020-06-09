use clap::{Arg, App};
use std::time::{Instant};
use rand::Rng;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::str;
use console::style;

const GRID_BLCK: usize = 3;
const GRID_SQRT: usize = GRID_BLCK * GRID_BLCK;
const GRID_SIZE: usize = GRID_SQRT * GRID_SQRT;
const NUM_TO_BITMAP: [usize;10] = [
    0b_000000000000, 
    0b_000000000001, 
    0b_000000000010, 
    0b_000000000100, 
    0b_000000001000, 
    0b_000000010000, 
    0b_000000100000, 
    0b_000001000000, 
    0b_000010000000, 
    0b_000100000000,
];

fn main() {
    // program start //
    let now = Instant::now();
    let app = App::new("Sudoko CLI - Quick!")
        .version("0.1.0")
        .author("Andre Sharpe <andre.sharpe@gmail.com>")
        .about("Solves and generates Sudoku puzzles, but fast!")
        .arg(Arg::with_name("solve")
            .short("s")
            .long("solve")
            .takes_value(false)
            .conflicts_with("generate")
            .help("Solves puzzles in a text file"))
        .arg(Arg::with_name("generate")
            .short("g")
            .long("generate")
            .takes_value(false)
            .conflicts_with("solve")
            .help("Generates puzzles and appends them to a text file"))
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .takes_value(true)
            .help("A file containing puzzles, one per line. Defaults to .\\puzzles.txt"))
        .arg(Arg::with_name("number")
            .short("n")
            .conflicts_with("solve")
            .long("number")
            .takes_value(true)
            .help("The number of puzzles to generate and append to file"))
        .arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .takes_value(false)
            .help("A file containing resulting solutions, one per line. Defaults to .\\puzzles.txt.solutions"));

    let matches = app.get_matches();
    let filename = matches.value_of("file").unwrap_or(".\\puzzle.txt");
    let output = matches.is_present("output");
    let count;
    if matches.is_present("generate") {
        let number = matches.value_of("number").unwrap_or("10").parse::<usize>().unwrap_or(10);
        count = Sudoku::generate_puzzles_to_file( &filename, number, output );
    } else {
         count = match Sudoku::solve_puzzles_from_file( &filename, output ) {
            Ok(number)  => number,
            Err(_e) => -1,
         }
    }

    let millisecs = now.elapsed().as_millis() as f64;
    let speed = f64::from( count )/(millisecs/1000.0f64);
    println!("Elapsed time: {:.3} seconds. Puzzles completed: {}. Peformance: {:.3} puzzles/second.", millisecs/1000.0f64, count, speed );
    // program end //
}

struct Sudoku {
    puzzle: [usize; GRID_SIZE],
    solutions: usize,
    limit: usize,
}

impl Sudoku {

    fn new() -> Sudoku {
        Sudoku {
            puzzle: [0; GRID_SIZE],
            solutions: 0,
            limit: 1,
        }
    }

    fn initialize_with_string( &mut self, str_puzzle: String ) {
        self.solutions = 0;
        let bytes = str_puzzle.as_bytes();
        if bytes.len() == GRID_SIZE {
            for (i,&b) in bytes.iter().enumerate() {
                if (b > b'0') && (b <= b'9') { self.puzzle[ i ] = (b - 48) as usize } else { self.puzzle[ i ] = 0 };
            }
        }
    }

    fn initialize_with_array( &mut self, a_puzzle: [usize;GRID_SIZE] ) {
        self.solutions = 0;
        for (i,&v) in a_puzzle.iter().enumerate() {
            self.puzzle[ i ] = v;
        }
    }

    fn clear( &mut self ) {
        self.solutions = 0;
        for i in 0..GRID_SIZE { self.puzzle[i] = 0; }
    }

    fn to_string( &self ) -> String {
        let mut x_puzzle: [u8;GRID_SIZE] = [0; GRID_SIZE];
        for i in 0..GRID_SIZE{
            x_puzzle[i] = (self.puzzle[i] + 48) as u8;
        }
        let s_puzzle = str::from_utf8(&x_puzzle).unwrap();
        String::from(s_puzzle)
    }
    
    fn solve_puzzles_from_file( filename: &str, output: bool ) -> io::Result<i32> {
        let puzzle_file = File::open( filename )?;
        let puzzle_file = BufReader::new( puzzle_file );
        let mut sudoku = Sudoku::new();
        let mut result = 0;

        if output { fs::remove_file(format!("{}{}", filename, ".solutions")).ok(); }
        for line in puzzle_file.lines() {
            let str_puzzle = line.unwrap();
            if str_puzzle.len() == GRID_SIZE {
                sudoku.initialize_with_string( str_puzzle );
                if (result) % 200 == 0 { 
                    println!( "Solving puzzle #{}", result+1 );
                    sudoku.display();
                };
                sudoku.solve_fast( 1 );
                if sudoku.solutions == 1 {
                    if (result) % 200 == 0 {
                        sudoku.display();
                    }
                } else {
                    println!( "There is no solution for this puzzle.");
                }
                if output {
                    sudoku.output_solution_to_file( &filename, result > 0 );
                }
                result += 1;
            }
        }
        Ok(result)
    }

    fn generate_puzzles_to_file( filename: &str, number: usize, output: bool ) -> i32 {
        let mut result = 0;
        let puzzle_file_exist = std::path::Path::new( filename ).exists();
        let mut puzzle_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(filename)
            .unwrap();
        let mut sudoku = Sudoku::new();
        for i in 0..number {
            sudoku.generate();
            if (result) % 100 == 0 { 
                println!("Generating puzzle {} of {}:", i+1, number );
                sudoku.display();
            }
            if puzzle_file_exist || result > 0 { puzzle_file.write_all("\n".as_bytes()).expect("Write failed."); }
            puzzle_file.write_all(sudoku.to_string().as_bytes()).expect("Write failed.");
            if output {
                sudoku.solve_fast( 1 );
                sudoku.output_solution_to_file( &filename, true );
            }
            result += 1;
        }
        return result;
    }

    fn output_solution_to_file( &self, filename: &str, newline: bool ) {
        let mut solution_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(format!("{}{}", filename, ".solutions"))
            .unwrap();
        let s_puzzle;
        if self.solutions == 0 {
            s_puzzle = std::iter::repeat(".").take( GRID_SIZE ).collect::<String>();
        } else {
            s_puzzle = self.to_string();
        }
        if newline { solution_file.write_all("\n".as_bytes()).expect("Write failed.") }; 
        solution_file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
    }
    
    fn display( &self ) {
        println!( "{}", style(" ┌─────────┬─────────┬─────────┐ ").green() ) ; 
        for i in 0..GRID_SIZE {
            if i % GRID_SQRT == 0  { print!("{}", style(" │").green()); }        
            if self.puzzle[i] == 0 { print!("{}", style(" . ").green()) } else { print!(" {} ", style(self.puzzle[i]).yellow().bright()) };
            let i1 = i+1;
            if i1 % GRID_BLCK == 0 { print!("{}", style("│").green()); }      
            if i1 != GRID_SIZE {                            
                if i1 % GRID_SQRT == 0  { println!(); }  
                if i1 % (GRID_SQRT*GRID_BLCK) == 0 { 
                    println!("{}", style(" ├─────────┼─────────┼─────────┤ ").green() ); 
                } 
            }   
        }
        println!();
        println!( "{}", style(" └─────────┴─────────┴─────────┘ ").green() );
        println!();
    }

    fn solve_fast( &mut self, limit: usize) {
        self.solutions = 0;
        self.limit = limit;
        self.solve_recursive_fast();
    }

    fn solve_random( &mut self, limit: usize) {
        self.solutions = 0;
        self.limit = limit;
        self.solve_recursive_random();
    }

    fn solve_recursive_fast( &mut self ) { 
        for i in 0..GRID_SIZE {
            if self.puzzle[ i ] == 0 {
                let b: usize = self.invalid_values_as_bits( i );
                for value in 1..GRID_SQRT+1 {
                    if  ( b & NUM_TO_BITMAP[ value ] ) == 0 {
                        self.puzzle[ i ] = value;
                        self.solve_recursive_fast();  // recurse!
                        if self.solutions == self.limit { return; }
                        self.puzzle[ i ] = 0;
                    }
                }
                return;
            }
        }
        self.solutions += 1;  // only reaches this point recursively when all cells are solved
    }
    
    fn solve_recursive_random( &mut self ) { 
        let mut numbers = [1,2,3,4,5,6,7,8,9];
        for i in 0..GRID_SIZE {
            if self.puzzle[i] == 0 {
                let b: usize = self.invalid_values_as_bits( i );
                Sudoku::shuffle(&mut numbers);
                for value in 0..GRID_SQRT {
                    if  ( b & NUM_TO_BITMAP[ numbers[ value ] ] ) == 0 {
                        self.puzzle[i] = numbers[ value ];
                        self.solve_recursive_random();  // recurse!
                        if self.solutions == self.limit { return; }
                        self.puzzle[ i ] = 0;
                    }
                }
                return;
            }
        }
        self.solutions += 1;  // only reaches this point recursively when all cells are solved
    }
    
    fn invalid_values_as_bits( &self, pos: usize ) -> usize {
        let y = pos / GRID_SQRT;
        let x = pos % GRID_SQRT;
        let topleft = ( y / GRID_BLCK ) * GRID_BLCK * GRID_SQRT + ( x / GRID_BLCK ) * GRID_BLCK; 
        let mut bits: usize = 0;
        for n in 0..GRID_SQRT {
            bits = bits 
                | NUM_TO_BITMAP[ self.puzzle[ n * GRID_SQRT + x ] ]  // check column
                | NUM_TO_BITMAP[ self.puzzle[ y * GRID_SQRT + n ] ]  // check row
                | NUM_TO_BITMAP[ self.puzzle[ topleft + ( n % GRID_BLCK ) * GRID_SQRT + ( n / GRID_BLCK  ) ] ] ; // check block
        }
        return bits;
    }

    fn generate( &mut self ) {

        // generate a random solution
        self.clear();
        self.solve_random( 1 );

        // create a new board and poulate with solution 
        let mut new = Sudoku::new();
        new.initialize_with_array( self.puzzle );
    
        // list to randomly remove numbers from solved board
        let mut removelist: [usize;GRID_SIZE] = [0; GRID_SIZE];
        for i in 0..GRID_SIZE { removelist[i] = i; }
        Sudoku::shuffle(&mut removelist);
    
        // randomly remove a number and confirm there is only one solution all the way or reverse it
        for i in 0..GRID_SIZE { 
            let save_item = new.puzzle[ removelist[i] ];
            new.puzzle[ removelist[i] ] = 0;
            self.initialize_with_array( new.puzzle );
            self.solve_fast( 2 );
            if self.solutions != 1 {
                new.puzzle[ removelist[i] ] = save_item;
            }
        }
        // transfer values from the new puzzle
        self.initialize_with_array( new.puzzle );
    }

    fn shuffle<T>(v: &mut [T]) {
        let mut rng = rand::thread_rng();
        let len = v.len();
         for n in 0..len {
            let i = rng.gen_range(0, len - n);
            v.swap(i, len - n - 1);
        }
    }
}
