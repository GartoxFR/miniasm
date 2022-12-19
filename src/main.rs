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

    let input = fs::read_to_string(filename).unwrap();

    let (_, program) = parse_program(&input).unwrap();
    let compiled_program = compile(&program);

    let default_output = "a.hex".to_owned();
    let filename = args.get(2).unwrap_or(&default_output);
    let mut file = File::create(filename).unwrap();

    file.write_all(b"v2.0 raw\n").unwrap();
    for byte in compiled_program {
        file.write_fmt(format_args!("{:x}\n", byte)).unwrap();
    }
}
