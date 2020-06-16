use clap::{Arg, App};
use std::time::{Instant};
use rand::Rng;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use console::style;
use console::Term;

const GRID_BLCK: usize = 3;
const GRID_SQRT: usize = GRID_BLCK * GRID_BLCK;
const GRID_SIZE: usize = GRID_SQRT * GRID_SQRT;
const NUM_TO_BITMAP: [usize;26] = [
    0b_0000000000000000000000000, 
    0b_0000000000000000000000001, 
    0b_0000000000000000000000010, 
    0b_0000000000000000000000100, 
    0b_0000000000000000000001000, 
    0b_0000000000000000000010000, 
    0b_0000000000000000000100000, 
    0b_0000000000000000001000000, 
    0b_0000000000000000010000000, 
    0b_0000000000000000100000000,
    0b_0000000000000001000000000,
    0b_0000000000000010000000000,
    0b_0000000000000100000000000,
    0b_0000000000001000000000000,
    0b_0000000000010000000000000,
    0b_0000000000100000000000000,
    0b_0000000001000000000000000,
    0b_0000000010000000000000000,
    0b_0000000100000000000000000,
    0b_0000001000000000000000000,
    0b_0000010000000000000000000,
    0b_0000100000000000000000000,
    0b_0001000000000000000000000,
    0b_0010000000000000000000000,
    0b_0100000000000000000000000,
    0b_1000000000000000000000000,
];
const NUM_TO_TEXT: [char;17] = ['.','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F','0'];

fn main() {
    // program start //
    let app = App::new("SUDOKU CLI Solver & Generator")
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
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .takes_value(false)
            .help("Interactively debug the solve steps for puzzles."))
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .help("Show solving steps in debug mode."))
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
            .help("A file containing solutions, one per line. Defaults to .\\puzzles.txt.solutions"));

    let matches = app.get_matches();
    let filename = String::from( matches.value_of("file").unwrap_or(".\\puzzle.txt") );
    let output_solutions = matches.is_present("output");
    let number = matches.value_of("number").unwrap_or("10").parse::<usize>().unwrap_or(10);
    let debug = matches.is_present("debug");
    let generate = matches.is_present("generate");
    let verbose = matches.is_present("verbose");
    let mut solutions_filename = filename.clone();
    solutions_filename.push_str(".solutions");
    let app_options = AppOptions{ filename, solutions_filename, output_solutions, number, debug, generate, verbose };

    let banner =
r" __           _       _          
/ _\_   _  __| | ___ | | ___   _ 
\ \| | | |/ _` |/ _ \| |/ / | | |
_\ \ |_| | (_| | (_) |   <| |_| |
\__/\__,_|\__,_|\___/|_|\_\\__,_|";

    println!("{}",style(banner).green().bright());
    println!("");
    println!("{}",style("SUDOKU CLI Solver & Generator").green().bright());
    println!("{}",style(" made with Rust in 2020").white());
    println!("");
    println!(" {} {}", style("build version.....").white(), style( format!( "{}x{}", GRID_BLCK, GRID_BLCK ) ).green() );
    println!(" {} {}", style("mode..............").white(), style( if app_options.generate { "generate" } else { "solve" }).green() );
    if app_options.generate { println!(" {} {}", style("number of puzzles.").white(), style(app_options.number ).green()) }
    println!(" {} {}", style("debug.............").white(), style(if app_options.debug { "yes" } else { "no" }).green() );
    if app_options.debug { println!(" {} {}", style("verbose output....").white(), style(if app_options.verbose { "yes" } else { "no" }).green()) }
    println!(" {} {}", style("puzzle file.......").white(), style(app_options.filename.clone()).green() );
    if app_options.output_solutions { println!(" {} {}", style("solutions file....").white(), style(app_options.solutions_filename.clone()).green() ) }
    println!("");

    let now = Instant::now();
    let count = Sudoku::run( app_options );
    let millisecs = now.elapsed().as_millis() as f64;
    let speed = f64::from( count )/(millisecs/1000.0f64);
    let line = format!("Elapsed time: {:.3} seconds. Puzzles completed: {}. Peformance: {:.3} puzzles/second.", millisecs/1000.0f64, count, speed );
    println!("{}",style(line).white());
    let term = Term::stdout();
    term.show_cursor().ok();
}

#[derive(Clone, Debug)]
struct AppOptions {
    filename: String,
    solutions_filename: String,
    output_solutions: bool,
    number: usize,
    debug: bool,
    generate: bool,
    verbose: bool,
}


struct Sudoku {
    puzzle: [usize; GRID_SIZE],
    markup: [usize; GRID_SIZE],
    solution: [usize; GRID_SIZE],
    solution_count: usize,
    limit: usize,
    app_options: AppOptions,
}

impl Sudoku {

    fn run( app_options: AppOptions ) -> i32 {
        let generate = app_options.generate;
        let mut sudoku = Sudoku::new( app_options );
        let count: i32;
        
        if generate {
            count = sudoku.generate_puzzles_to_file();
        } else {
             count = match sudoku.solve_puzzles_from_file() {
                Ok(number)  => number,
                Err(_e) => -1,
             }
        }
        count
    }

    fn new( app_options: AppOptions ) -> Sudoku {
        Sudoku {
            puzzle: [0 ; GRID_SIZE],
            markup: [0 ; GRID_SIZE],
            solution: [0; GRID_SIZE],
            solution_count: 0,
            limit: 1,
            app_options,
        }
    }

    fn initialize_with_string( &mut self, str_puzzle: String ) {
        let bytes = str_puzzle.as_bytes();
        let mut a_puzzle: [usize;GRID_SIZE] = [0;GRID_SIZE];
        if bytes.len() == GRID_SIZE {
            for (pos,&b) in bytes.iter().enumerate() {
                if (b >= b'1') && (b <= b'9') {
                    a_puzzle[ pos ] = (b - 48) as usize 
                } else if (b >= b'A') && (b <= b'F') {
                    a_puzzle[ pos ] = (b - 55) as usize 
                } else if b == b'0' {
                    a_puzzle[ pos ] = 16 
                } else { 
                    a_puzzle[ pos ] = 0 
                };
            }
            self.initialize_with_array( a_puzzle );
        }
    }

    fn initialize_with_array( &mut self, a_puzzle: [usize;GRID_SIZE] ) {
        self.clear();
        for (pos,&val) in a_puzzle.iter().enumerate() {
            self.puzzle[ pos ] = val;
            self.solution[ pos ] = val;
        }
    }

    fn clear( &mut self ) {
        self.solution_count = 0;
        for pos in 0..GRID_SIZE { self.puzzle[ pos ] = 0; self.solution[ pos ] = 0; self.markup[ pos ] = usize::MAX; }
    }

    fn do_markup( &mut self ) {
        for pos in 0..GRID_SIZE {
            if self.solution[ pos ] == 0 {
                self.markup[ pos ] = self.invalid_values_as_bits(pos);
            } else {
                self.markup[ pos ] = 0; // fill with 1's - all values invalid
            }
        }
    }

    fn set_value_and_markup( &mut self, pos: usize, value: usize ){
        let y = pos / GRID_SQRT;
        let x = pos % GRID_SQRT;
        let topleft = ( y / GRID_BLCK ) * GRID_BLCK * GRID_SQRT + ( x / GRID_BLCK ) * GRID_BLCK; 
        self.solution[ pos ] = value;
        let bitmap = NUM_TO_BITMAP[ value ];
        for n in 0..GRID_SQRT {
            self.markup[ n * GRID_SQRT + x ] |= bitmap; 
            self.markup[ y * GRID_SQRT + n ] |= bitmap; 
            self.markup[ topleft + ( n % GRID_BLCK ) * GRID_SQRT + ( n / GRID_BLCK  ) ] |= bitmap;
        }
        self.markup[ pos ] = usize::MAX;
    }

    fn to_string( &self ) -> String {
        let mut s_puzzle = String::new();
        for pos in 0..GRID_SIZE{
            s_puzzle.push( NUM_TO_TEXT[ self.solution[pos] ] );
        }
        s_puzzle
    }
    
    fn solve_puzzles_from_file( &mut self ) -> io::Result<i32> {
        let filename = self.app_options.filename.clone();
        let solution_filename = self.app_options.solutions_filename.clone();
        let puzzle_file = File::open( &filename )?;
        let puzzle_file = BufReader::new( puzzle_file );
        let mut result = 0;
        let mut solution_file;

        if self.app_options.output_solutions { 
            fs::remove_file(&solution_filename).ok(); 
        }
        solution_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&solution_filename)
            .unwrap();

        for line in puzzle_file.lines() {
            let str_puzzle = line.unwrap();
            if str_puzzle.len() == GRID_SIZE {
                self.initialize_with_string( str_puzzle );
                if self.app_options.debug {
                    self.solution_count = 1;
                    self.display( format!("Attempting puzzle #{}...", result+1) );
                    self.solution_count = 0;
                }
                self.solve_fast( 1 );
                if self.solution_count == 1 {
                    if self.app_options.debug {
                        self.display( format!("...solved puzzle #{}", result+1) );
                    }
                } else {
                    println!( "There is no solution for puzzle #{}.", result+1);
                }
                if self.app_options.output_solutions {
                    // self.output_solution_to_file( result > 0 );
                    let s_puzzle;
                    if self.solution_count == 0 {
                        s_puzzle = std::iter::repeat(".").take( GRID_SIZE ).collect::<String>();
                    } else {
                        s_puzzle = self.to_string();
                    }
                    if result > 0 { solution_file.write_all("\n".as_bytes()).expect("Write failed.") }; 
                    solution_file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
                }
                result += 1;
            }
        }
        Ok(result)
    }

    fn generate_puzzles_to_file( &mut self ) -> i32 {
        let filename = self.app_options.filename.clone();
        let mut result = 0;
        let puzzle_file_exist = std::path::Path::new( &filename ).exists();
        let mut buffer = String::with_capacity((GRID_SIZE+1) * self.app_options.number);
        let mut puzzle_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&filename)
            .unwrap();
        let mut solution_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(self.app_options.solutions_filename.clone())
            .unwrap();
        for i in 0..self.app_options.number {
            self.generate();
            if self.app_options.debug { 
                self.display( format!("...generated puzzle {} of {}:", i+1, self.app_options.number ) );
            }
            if puzzle_file_exist || result > 0 { 
                buffer += "\n"; 
            }
            buffer += &self.to_string();
            if self.app_options.output_solutions {
                self.solve_fast( 1 );
                let s_puzzle = self.to_string();
                solution_file.write_all("\n".as_bytes()).expect("Write failed."); 
                solution_file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
            }
            result += 1;
        }
        puzzle_file.write_all(buffer.as_bytes()).expect("Write failed.");
        result
    }

    fn display( &self, heading: String ) {
        let term = Term::stdout();
        term.hide_cursor().ok();
        println!( "{}", style( heading ).white() ) ; 

        let segment = std::iter::repeat("─").take( GRID_BLCK*3 ).collect::<String>(); 
        let mut line = String::new();
        line += " ┌";
        line += &segment;
        for _i in 0..GRID_BLCK-1 {
            line += "┬";
            line += &segment;
        }
        line += "┐ ";
        println!( "{}", style( &line ).green() ) ; 

        let mut line = String::new();
        line += " ├";
        line += &segment;
        for _i in 0..GRID_BLCK-1 {
            line += "┼";
            line += &segment;
        }
        line += "┤ ";

        for i in 0..GRID_SIZE {
            if i % GRID_SQRT == 0  { print!("{}", style(" │").green()); }        
            if self.puzzle[i] == 0 {
                print!(" {} ", style( NUM_TO_TEXT[ self.solution[i] ] ).yellow());
            } else {
                print!(" {} ", style( NUM_TO_TEXT[ self.solution[i] ] ).yellow().bright());
            }
            let i1 = i+1;
            if i1 % GRID_BLCK == 0 { print!("{}", style("│").green() ); }      
            if i1 != GRID_SIZE {                            
                if i1 % GRID_SQRT == 0  { println!(); }  
                if i1 % (GRID_SQRT*GRID_BLCK) == 0 { 
                   println!("{}", style( &line ).green() ); 
                } 
            }   
        }

        let mut line = String::new();
        line += " └";
        line += &segment;
        for _i in 0..GRID_BLCK-1 {
            line += "┴";
            line += &segment;
        }
        line += "┘ ";

        println!();
        println!( "{}", style( &line ).green() );
        println!();
        if self.solution_count != self.limit {
            term.move_cursor_up( GRID_SQRT+GRID_BLCK+3 ).ok();
            term.show_cursor().ok();
        }
    }

    fn solve_fast( &mut self, limit: usize) {
        self.solution_count = 0;
        self.limit = limit;
        self.solve_lonerangers();
        self.solve_recursive_fast();
    }

    fn solve_random( &mut self, limit: usize) {
        self.solution_count = 0;
        self.limit = limit;
        self.solve_recursive_random();
    }

    // solves easy cells 
    fn solve_lonerangers( &mut self ) {
        let mut r_solved; // row
        let mut c_solved; // column
        let mut b_solved; // block

        self.do_markup();
        loop {

            r_solved = 0;
            for value in 1..GRID_SQRT+1 {
                let bitmap = NUM_TO_BITMAP[ value ];
                for r in 0..GRID_SQRT {
                    let mut count = 0;
                    let mut pos = 0;
                    for c in 0..GRID_SQRT {
                        let p = r*GRID_SQRT + c;
                        if self.solution[ p ] == 0 && (( self.markup[ p ] & bitmap ) == 0) {
                            count+= 1;
                            if count > 1 { break; }
                            pos = p;
                        }
                    }
                    if count == 1 {
                        self.set_value_and_markup(pos, value);
                        r_solved += 1;
                    }
                }
            }

            c_solved = 0;
            for value in 1..GRID_SQRT+1 {
                let bitmap = NUM_TO_BITMAP[ value ];
                for c in 0..GRID_SQRT {
                    let mut count = 0;
                    let mut pos = 0;
                    for r in 0..GRID_SQRT {
                        let p = r*GRID_SQRT + c;
                        if self.solution[ p ] == 0 && (( self.markup[ p ] & bitmap ) == 0) {
                            count+= 1;
                            if count > 1 { break; }
                            pos = p;
                        }
                    }
                    if count == 1 {
                        self.set_value_and_markup(pos, value);
                        c_solved += 1;
                    }
                }
            }

            b_solved = 0;
            for value in 1..GRID_SQRT+1 {
                let bitmap = NUM_TO_BITMAP[ value ];
                for b in 0..GRID_SQRT {
                    let mut count = 0;
                    let mut pos = 0;
                    let tl = (b/GRID_BLCK)*GRID_SQRT*GRID_BLCK + (b % GRID_BLCK)*GRID_BLCK;
                    for r in 0..GRID_BLCK {
                        for c in 0..GRID_BLCK {
                            let p = tl + r*GRID_SQRT + c;
                            if self.solution[ p ] == 0 && (( self.markup[ p ] & bitmap ) == 0) {
                                count+= 1;
                                if count > 1 { break; }
                                pos = p;
                            }
                            if count > 1 { break; }
                        }
                    }
                    if count == 1 {
                        self.set_value_and_markup(pos, value);
                        b_solved += 1;
                    }
                }
            }

            if r_solved + c_solved + b_solved == 0 {
                break;
            }

        }

    }

    fn solve_recursive_fast( &mut self ) { 
        if self.app_options.verbose && self.app_options.debug { 
            self.display( format!("....solving......") );
        }
        for pos in 0..GRID_SIZE {
            if self.solution[ pos ] == 0 {
                let b = self.invalid_values_as_bits(pos);
                for value in 1..GRID_SQRT+1 {
                    if  ( b & NUM_TO_BITMAP[ value ] ) == 0 {
                        self.solution[ pos ] = value;
                        self.solve_recursive_fast();  // recurse!
                        if self.solution_count == self.limit { return; }
                        self.solution[ pos ] = 0;
                    }
                }
                return;
            }
        }
        self.solution_count += 1;  // only reaches this point recursively when all cells are solved
    }
    
    fn solve_recursive_random( &mut self ) { 
        let mut numbers: [usize; GRID_SQRT] = [0; GRID_SQRT];
        for pos in 0..GRID_SQRT { numbers[pos] = pos+1 }
        for pos in 0..GRID_SIZE {
            if self.solution[ pos ] == 0 {
                Sudoku::shuffle(&mut numbers);
                let b = self.invalid_values_as_bits(pos);
                for value in 0..GRID_SQRT {
                    if  ( b & NUM_TO_BITMAP[ numbers[ value ] ] ) == 0 {
                        self.solution[ pos ] = numbers[ value ];
                        self.solve_recursive_random();  // recurse!
                        if self.solution_count == self.limit { return; }
                        self.solution[ pos ] = 0;
                    }
                }
                return;
            }
        }
        self.solution_count += 1;  // only reaches this point recursively when all cells are solved
    }

    fn invalid_values_as_bits( &self, pos: usize ) -> usize {
        let y = pos / GRID_SQRT;
        let x = pos % GRID_SQRT;
        let topleft = ( y / GRID_BLCK ) * GRID_BLCK * GRID_SQRT + ( x / GRID_BLCK ) * GRID_BLCK; 
        let mut bits: usize = 0;
        for n in 0..GRID_SQRT {
            bits = bits 
                | NUM_TO_BITMAP[ self.solution[ n * GRID_SQRT + x ] ]  // check column
                | NUM_TO_BITMAP[ self.solution[ y * GRID_SQRT + n ] ]  // check row
                | NUM_TO_BITMAP[ self.solution[ topleft + ( n % GRID_BLCK ) * GRID_SQRT + ( n / GRID_BLCK  ) ] ] ; // check block
        }
        bits
    }

    fn generate( &mut self ) {

        // generate a random solution
        self.clear();
        self.solve_random( 1 );

        // copy solution 
        let mut new_puzzle: [usize;GRID_SIZE] = [0; GRID_SIZE];
        for i in 0..GRID_SIZE { new_puzzle[i] = self.solution[i]; }
    
        // list to randomly remove numbers from solved board
        let mut removelist: [usize;GRID_SIZE] = [0; GRID_SIZE];
        for i in 0..GRID_SIZE { removelist[i] = i; }
        Sudoku::shuffle(&mut removelist);
    
        // randomly remove a number and confirm there is only one solution all the way or reverse it
        for i in 0..GRID_SIZE { 
            let save_item = new_puzzle[ removelist[i] ];
            new_puzzle[ removelist[i] ] = 0;
            self.initialize_with_array( new_puzzle );
            if self.app_options.debug { 
                self.display( format!("Removing {} : {}   ", i, removelist[i]) );
            }
            // let now = Instant::now();
            self.solve_fast( 2 );
            if self.solution_count != 1 {
                new_puzzle[ removelist[i] ] = save_item;
            }
            // if !self.app_options.debug && (now.elapsed().as_millis() > 1000) {
            //     break;
            // }
        }
        // transfer values from the new puzzle
        if self.app_options.debug { 
            self.solution_count = 1;
            self.limit = 1;
            self.display( format!("With solution...              ") );
        }
        self.initialize_with_array( new_puzzle );
        if self.app_options.debug { 
            self.solution_count = 1
        }
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
