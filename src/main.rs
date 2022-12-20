mod compiler;
mod parser;
mod types;

use std::fs::File;
use std::io::Write;
use std::{env, fs};

use self::compiler::compile;
use self::parser::parse_program;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).expect("Expected 1 argument : file to compile");

    let input = match fs::read_to_string(filename) {
        Ok(input) => input,
        Err(e) => {
            eprintln!("An error has occured while reading input file : {}", e);
            return;
        }
    };

    let (_, program) = match parse_program(&input) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("An error has occured while parsing input file : {}", e);
            return;
        }
    };

    let compiled_program = match compile(&program) {
        Ok(cp) => cp,
        Err(e) => {
            eprintln!("Compiler error : {}", e.0);
            return;
        }
    };

    let default_output = "a.hex".to_owned();
    let filename = args.get(2).unwrap_or(&default_output);

    let mut file = match File::create(filename) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("An error has occured while creating output file : {}", e);
            return;
        }
    };

    if let Err(e) = file.write_all(b"v2.0 raw\n") {
        eprintln!("An error has occured while writing to output file : {}", e);
        return;
    }
    for byte in compiled_program {
        if let Err(e) = file.write_fmt(format_args!("{:x}\n", byte)) {
            eprintln!("An error has occured while writing to output file : {}", e);
            return;
        }
    }
}
