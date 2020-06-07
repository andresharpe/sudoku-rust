use clap::{Arg, App};
use std::time::{Instant};
use rand::Rng;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;

use std::str;

const GRID_BLCK: usize = 3;
const GRID_SQRT: usize = GRID_BLCK * GRID_BLCK;
const GRID_SIZE: usize = GRID_SQRT*GRID_SQRT;

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
                 .help("The number of puzzles to generate and append to file"));

    let matches = app.get_matches();
    let filename = matches.value_of("file").unwrap_or(".\\puzzle.txt");
    let count;

    if matches.is_present("generate") {
        let number = matches.value_of("number").unwrap_or("10").parse::<u32>().unwrap_or(10);
        count = puzzle_generate_file( &filename, number );
    } else {
         count = match puzzle_solve_file( &filename ) {
            Ok(number)  => number,
            Err(_e) => -1,
         }
    }

    let secs = now.elapsed().as_secs() as f64;
    let speed = f64::from( count )/secs;
    println!("Elapsed time: {} seconds. Puzzles completed: {}. Peformance: {:.3} puzzles/second.", secs, count, speed );

}

fn puzzle_solve_file( filename: &str ) -> io::Result<i32> {
    let f = File::open( filename )?;
    let f = BufReader::new(f);
    let mut puzzle: [u8; GRID_SIZE] = [0; GRID_SIZE];
    let mut result = 0;

    for line in f.lines() {
        let str_puzzle = line.unwrap();
        if str_puzzle.len() > GRID_SIZE-1 {
            puzzle_from_string( &mut puzzle, str_puzzle );
            if (result) % 200 == 0 { 
                println!( "Solving puzzle #{}", result+1 );
                puzzle_output( &mut puzzle );
            };
            let solutions = puzzle_solve_all( &mut puzzle, 1, false );
            if solutions == 1{
                if (result) % 200 == 0 {
                    puzzle_output( &mut puzzle );
                }
            } else {
                println!();
                println!( "There is no solution for this puzzle.");
            }
            result += 1;
        }
    }
    Ok(result)
}

fn puzzle_generate_file( filename: &str, number: u32 ) -> i32 {
    let mut puzzle: [u8; GRID_SIZE] = [0; GRID_SIZE];
    let mut result = 0;
    let mut file = OpenOptions::new()
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
        file.write_all("\n".as_bytes()).expect("Write failed.");
        file.write_all(s_puzzle.as_bytes()).expect("Write failed.");
        result += 1;
    }
    return result;
}


fn puzzle_from_string( puzzle: &mut [u8; GRID_SIZE], str_puzzle: String ) {
    let bytes = str_puzzle.as_bytes();
    let mut i : usize = 0;
    if bytes.len() > GRID_SIZE-1 {
        for b in bytes.iter() {
            puzzle[ i ] = b - 48; // u8::from( b - 48 );
            i += 1;
        }
    }
}

fn puzzle_to_string( puzzle: &mut [u8; GRID_SIZE] ) -> String {
    let mut x_puzzle: [u8;GRID_SIZE] = [0; GRID_SIZE];
    for i in 0..GRID_SIZE{
        x_puzzle[i] = puzzle[i] + 48;
    }
    let s_puzzle = str::from_utf8(&x_puzzle).unwrap();
    return String::from(s_puzzle);
}

fn puzzle_output( puzzle: &mut [u8; GRID_SIZE] ) {
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

fn puzzle_invalid_values_as_bits( puzzle: &mut [u8; GRID_SIZE], pos: usize ) -> u32 {
    let y = pos / GRID_SQRT;
    let x = pos % GRID_SQRT;
    let topleft = ( y / GRID_BLCK ) * GRID_BLCK * 9 + ( x / GRID_BLCK ) * GRID_BLCK; 
    let mut bits: u32 = 0;
    for n in 0..GRID_SQRT {
        let b = puzzle[ n * GRID_SQRT + x ];
        if b > 0 { bits = bits | 1 << (b-1); }
        let b = puzzle[ y * GRID_SQRT + n ];
        if b > 0 { bits = bits | 1 << (b-1);  }
        let b = puzzle[ topleft + ( n % GRID_BLCK ) * GRID_SQRT + ( n / GRID_BLCK  ) ];
        if b > 0 { bits = bits | 1 << (b-1); }
    }
    return bits;
}

fn puzzle_solve_all( puzzle: &mut [u8; GRID_SIZE], limit: u8, random: bool ) -> u8 {
    let mut solutions = 0;
    if random {
        puzzle_solve_random( puzzle, limit, &mut solutions ); // slower, but needed for generating new puzzles
    } else {
        puzzle_solve( puzzle, limit, &mut solutions ); 
    }
    return solutions;
}

fn puzzle_solve( puzzle: &mut [u8; GRID_SIZE], limit: u8, solutions: &mut u8 ) { 
    for i in 0..GRID_SIZE {
        if puzzle[i] == 0 {
            let b: u32 = puzzle_invalid_values_as_bits( puzzle, i );
            for value in 0..GRID_SQRT {
                if  ( b >> value ) & 1 == 0 {
                    puzzle[i] = (value+1) as u8;
                    puzzle_solve( puzzle, limit, solutions );  // recurse!
                    if *solutions == limit { return; }
                    puzzle[i] = 0;
                }
            }
            return;
        }
    }
    *solutions += 1;  // only reaches this point recursively when all cells are solved
}

fn puzzle_solve_random( puzzle: &mut [u8; GRID_SIZE], limit: u8, solutions: &mut u8 ) { 
    let mut numbers = [1,2,3,4,5,6,7,8,9];
    for i in 0..GRID_SIZE {
        if puzzle[i] == 0 {
            let b: u32 = puzzle_invalid_values_as_bits( puzzle, i );
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

fn puzzle_generate( puzzle: &mut [u8; GRID_SIZE] ) {

    // generate a random solution
    for i in 0..GRID_SIZE { puzzle[i] = 0; }
    puzzle_solve_all( puzzle, 1, true );

    let mut new_puzzle: [u8;81] = [0; GRID_SIZE];
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

    // done
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