use clap::{Arg, App};
use std::time::{Instant};
use rand::Rng;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;

use std::str;

const GRID_BLCK: usize = 3;
const GRID_SQRT: usize = GRID_BLCK * GRID_BLCK;
const GRID_SIZE: usize = GRID_SQRT * GRID_SQRT;
const NUM_BITMAP: [usize;10] = [ 0b000000000, 0b000000001, 0b000000010, 0b000000100, 0b000001000, 0b000010000, 0b000100000, 0b001000000, 0b010000000, 0b100000000 ];

fn main() {
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
        count = puzzle_generate_file( &filename, number, output );
    } else {
         count = match puzzle_solve_file( &filename, output ) {
            Ok(number)  => number,
            Err(_e) => -1,
         }
    }

    let secs = now.elapsed().as_secs() as f64;
    let speed = f64::from( count )/secs;
    println!("Elapsed time: {} seconds. Puzzles completed: {}. Peformance: {:.3} puzzles/second.", secs, count, speed );

}

fn puzzle_solve_file( filename: &str, output: bool ) -> io::Result<i32> {
    let puzzle_file = File::open( filename )?;
    let puzzle_file = BufReader::new( puzzle_file );
    let mut puzzle: [usize; GRID_SIZE] = [0; GRID_SIZE];
    let mut result = 0;
    if output {
        fs::remove_file(format!("{}{}", filename, ".solutions")).ok();
    }
    for line in puzzle_file.lines() {
        let str_puzzle = line.unwrap();
        if str_puzzle.len() == GRID_SIZE {
            let s_original = if output { str_puzzle.clone() } else { String::new() };
            puzzle_from_string( &mut puzzle, str_puzzle );
            if (result) % 200 == 0 { 
                println!( "Solving puzzle #{}", result+1 );
                puzzle_output( &mut puzzle );
            };
            let solutions = puzzle_solve_all( &mut puzzle, 1, false );
            if solutions == 1 {
                if (result) % 200 == 0 {
                    puzzle_output( &mut puzzle );
                }
            } else {
                println!();
                println!( "There is no solution for this puzzle.");
            }
            if output {
                let mut solution_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(format!("{}{}", filename, ".solutions"))
                    .unwrap();
                if result > 0 { solution_file.write_all("\n".as_bytes()).expect("Write failed."); }
                if solutions == 1 {
                    let s_puzzle = puzzle_to_string( &mut puzzle );
                    solution_file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
                } else {
                    solution_file.write_all(s_original.as_bytes()).expect("Write failed.");
                }
            }
            result += 1;
        }
    }
    Ok(result)
}

fn puzzle_generate_file( filename: &str, number: usize, output: bool ) -> i32 {
    let mut puzzle: [usize; GRID_SIZE] = [0; GRID_SIZE];
    let mut result = 0;
    let mut puzzle_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    for i in 0..number {
        puzzle_generate( &mut puzzle );
        if (result) % 100 == 0 { 
            println!("Generating puzzle {} of {}:", i+1, number );
            puzzle_output( &mut puzzle );
        }
        let s_puzzle = puzzle_to_string( &mut puzzle );
        puzzle_file.write_all("\n".as_bytes()).expect("Write failed.");
        puzzle_file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
        if output {
            puzzle_solve_all( &mut puzzle, 1, false );
            let mut solution_file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(format!("{}{}", filename, ".solutions"))
                .unwrap();
            let s_puzzle = puzzle_to_string( &mut puzzle );
            solution_file.write_all("\n".as_bytes()).expect("Write failed."); 
            solution_file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
        }
        result += 1;
    }
    return result;
}

fn puzzle_from_string( puzzle: &mut [usize; GRID_SIZE], str_puzzle: String ) {
    let bytes = str_puzzle.as_bytes();
    let mut i : usize = 0;
    if bytes.len() == GRID_SIZE {
        for b in bytes.iter() {
            if (*b > b'0') && (*b <= b'9') { puzzle[ i ] = ( *b - 48 ) as usize } else { puzzle[ i ] = 0 as usize };
            i += 1; 
        }
    }
}

fn puzzle_to_string( puzzle: &mut [usize; GRID_SIZE] ) -> String {
    let mut x_puzzle: [u8;GRID_SIZE] = [0; GRID_SIZE];
    for i in 0..GRID_SIZE{
        x_puzzle[i] = (puzzle[i] + 48) as u8;
    }
    let s_puzzle = str::from_utf8(&x_puzzle).unwrap();
    return String::from(s_puzzle);
}

fn puzzle_output( puzzle: &mut [usize; GRID_SIZE] ) {
    println!( " +---------+---------+---------+ " ); 
    for i in 0..GRID_SIZE {
        if i % GRID_SQRT == 0  { print!(" |"); }        
        if puzzle[i] == 0 { print!(" . ") } else { print!(" {} ", puzzle[i] ) };
        let i1 = i+1;
        if i1 % GRID_BLCK == 0 { print!("|"); }      
        if i1 != GRID_SIZE {                            
            if i1 % GRID_SQRT == 0  { println!(); }  
            if i1 % (GRID_SQRT*GRID_BLCK) == 0 { 
                println!(" |---------+---------+---------| "); 
            } 
        }   
    }
    println!();
    println!( " +---------+---------+---------+ " );
    println!();
}

fn puzzle_invalid_values_as_bits( puzzle: &mut [usize; GRID_SIZE], pos: usize ) -> usize {
    let y = pos / GRID_SQRT;
    let x = pos % GRID_SQRT;
    let topleft = ( y / GRID_BLCK ) * GRID_BLCK * GRID_SQRT + ( x / GRID_BLCK ) * GRID_BLCK; 
    let mut bits: usize = 0;
    for n in 0..GRID_SQRT {
        bits = bits 
            | NUM_BITMAP[ puzzle[ n * GRID_SQRT + x ] ] 
            | NUM_BITMAP[ puzzle[ y * GRID_SQRT + n ] ] 
            | NUM_BITMAP[ puzzle[ topleft + ( n % GRID_BLCK ) * GRID_SQRT + ( n / GRID_BLCK  ) ] ] ;
    }
    return bits;
}

fn puzzle_solve_all( puzzle: &mut [usize; GRID_SIZE], limit: usize, random: bool ) -> usize {
    let mut solutions = 0;
    if random {
        puzzle_solve_random( puzzle, limit, &mut solutions ); // slower, but needed for generating new puzzles
    } else {
        puzzle_solve_fast( puzzle, limit, &mut solutions ); 
    }
    return solutions;
}

fn puzzle_solve_fast( puzzle: &mut [usize; GRID_SIZE], limit: usize, solutions: &mut usize ) { 
    for i in 0..GRID_SIZE {
        if puzzle[i] == 0 {
            let b: usize = puzzle_invalid_values_as_bits( puzzle, i );
            for value in 1..GRID_SQRT+1 {
                if  ( b & NUM_BITMAP[ value ] ) == 0 {
                    puzzle[i] = value;
                    puzzle_solve_fast( puzzle, limit, solutions );  // recurse!
                    if *solutions == limit { return; }
                    puzzle[i] = 0;
                }
            }
            return;
        }
    }
    *solutions += 1;  // only reaches this point recursively when all cells are solved
}

fn puzzle_solve_random( puzzle: &mut [usize; GRID_SIZE], limit: usize, solutions: &mut usize ) { 
    let mut numbers = [1,2,3,4,5,6,7,8,9];
    for i in 0..GRID_SIZE {
        if puzzle[i] == 0 {
            let b: usize = puzzle_invalid_values_as_bits( puzzle, i );
            shuffle(&mut numbers);
            for value in 0..GRID_SQRT {
                if  ( b >> numbers[value]-1 ) & 1 == 0 {
                    puzzle[i] = numbers[value];
                    puzzle_solve_random( puzzle, limit, solutions );  // recurse!
                    if *solutions >= limit { return; }
                    puzzle[i] = 0;
                }
            }
            return;
        }
    }
    *solutions += 1;  // only reaches this point recursively when all cells are solved
}

fn puzzle_generate( puzzle: &mut [usize; GRID_SIZE] ) {

    // generate a random solution
    for i in 0..GRID_SIZE { puzzle[i] = 0; }
    puzzle_solve_all( puzzle, 1, true );

    let mut new_puzzle: [usize;GRID_SIZE] = [0; GRID_SIZE];
    for i in 0..GRID_SIZE { new_puzzle[i] = puzzle[i]; }

    // remove numbers from solved board
    let mut removelist: [usize;GRID_SIZE] = [0; GRID_SIZE];
    for i in 0..GRID_SIZE { removelist[i] = i; }
    shuffle(&mut removelist);

    // systematically remove a number and confirm there is only one solution all the way or reverse it
    for i in 0..GRID_SIZE { 
        let save_item = new_puzzle[ removelist[i] ];
        new_puzzle[ removelist[i] ] = 0;
        for i in 0..GRID_SIZE { puzzle[i] = new_puzzle[i]; }
        let solutions = puzzle_solve_all( puzzle, 2, false );
        if solutions != 1 {
            new_puzzle[ removelist[i] ] = save_item;
        }
    }
   for i in 0..GRID_SIZE { puzzle[i] = new_puzzle[i]; }
}

fn shuffle<T>(v: &mut [T]) {
    let mut rng = rand::thread_rng();
    let len = v.len();
     for n in 0..len {
        let i = rng.gen_range(0, len - n);
        v.swap(i, len - n - 1);
    }
}