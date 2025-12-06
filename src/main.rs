mod compiler;

use crate::compiler::file::SourceFile;
use crate::compiler::Compiler;
use clap::Parser;
use std::io::Write;
use std::{fs, io};

#[derive(Parser)]
struct Args {
    #[arg(help = "the filename of the file to compile",required_unless_present_any = ["cli","version"])]
    input: Vec<String>,

    #[arg(long, short = 'd')]
    debug: bool,

    #[arg(long, help = "terminal mode")]
    cli: bool,

    #[arg(long, short = 'v')]
    version: bool,
}

fn main() {
    let args = Args::parse();
    let mut compiler = Compiler::new();

    if args.version {
        println!("{}", compiler.get_version());
        println!("Copyright 2023-2026 by MCPPL,DotCS");
        return ();
    }

    if args.cli {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                compiler.add_file(SourceFile::new("<console>".to_string(), input));
            }
            Err(_) => {
                eprintln!("error: cannot read from stdin.");
            }
        }
    } else {
        for file in args.input {
            let file_name = file.clone();
            let data =
                fs::read_to_string(file).unwrap_or_else(|e| panic!("error: cannot read file{}", e));
            compiler.add_file(SourceFile::new(file_name, data));
        }
    }

    compiler.compile();
}
