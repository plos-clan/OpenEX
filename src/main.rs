mod compiler;

use crate::compiler::file::SourceFile;
use crate::compiler::Compiler;
use getopts_macro::getopts_options;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::{fs, io};

struct Args {
    input: Vec<String>,
    #[expect(unused)]
    debug: bool,
    cli: bool,
    version: bool,
}

impl Args {
    fn parse() -> Self {
        let options = getopts_options! {
            -d, --debug     "";
                --cli       "terminal mode";
            -v, --version   "Print version";
            -h, --help*     "Print help";
        };
        let m = match options.parse(std::env::args().skip(1)) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                exit(2)
            },
        };
        if m.opt_present("help") {
            Self::help(&options);
            exit(1)
        }
        let args = Self {
            debug:   m.opt_present("debug"),
            cli:     m.opt_present("cli"),
            version: m.opt_present("version"),
            input:   m.free,
        };
        args.check();
        args
    }

    fn check(&self) {
        if self.input.is_empty()
            && !self.cli
            && !self.version
        {
            eprintln!("error: required arguments were not provided: <INPUT>...");
            exit(2)
        }
    }

    fn help(options: &getopts_macro::getopts::Options) {
        let brief = format!(
            "Usage: {} [OPTIONS] [INPUT]...\n\n\
            Arguments:\n  [INPUT]...  the filename of the file to compile",
            Self::prog_name(),
        );
        print!("{}", options.usage(&brief));
    }

    fn prog_name() -> String {
        std::env::args_os()
            .next()
            .and_then(|name| {
                PathBuf::from(name)
                    .file_name()?
                    .to_string_lossy()
                    .into_owned()
                    .into()
            })
            .unwrap_or_else(|| env!("CARGO_BIN_NAME").into())
    }
}

fn main() {
    let args = Args::parse();
    let mut compiler = Compiler::new();

    if args.version {
        println!("OpenEX RustEdition v{}", compiler.get_version());
        println!("Copyright 2023-2026 by MCPPL,DotCS");
        return ;
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
