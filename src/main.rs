use std::env;
use std::io;
use std::process;

mod regex_matcher;

use regex_matcher::match_pattern as basic_match_pattern;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} [-E|-e] <pattern>", args[0]);
        process::exit(1);
    }

    let pattern = &args[2];
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let result = basic_match_pattern(input_line.trim(), pattern);

    if result {
        println!("Code 0");
        process::exit(0);
    } else {
        println!("Code 1");
        process::exit(1);
    }
}