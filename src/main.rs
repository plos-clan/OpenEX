mod compiler;
mod library;
mod runtime;

use crate::compiler::file::SourceFile;
use crate::compiler::{lints, Compiler};
use crate::library::{load_libraries};
use getopts_macro::getopts_options;
use smol_str::SmolStr;
use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::{fs, io};
use crate::runtime::executor_run;

struct Args {
    input: Vec<String>,
    #[expect(unused)]
    debug: bool,
    cli: bool,
    allow: HashSet<lints::Lint>,
    lib: Option<SmolStr>,
    version: bool,
}

impl Args {
    fn parse() -> Self {
        let options = getopts_options! {
            -d, --debug         "";
                --cli           "terminal mode";
            -A, --allow*=LINT   "Disable compiler warning";
            -v, --version       "Print version";
            -h, --help*         "Print help";
            -l, --lib*          "Set libraries directory";
        };
        let m = match options.parse(std::env::args().skip(1)) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                exit(2)
            }
        };
        if m.opt_present("help") {
            Self::help(&options);
            exit(1)
        }
        let args = Self {
            debug: m.opt_present("debug"),
            cli: m.opt_present("cli"),
            allow: m
                .opt_strs("allow")
                .iter()
                .filter_map(Self::parse_allow)
                .collect(),
            version: m.opt_present("version"),
            lib: m
                .opt_strs("lib")
                .iter()
                .find_map(Self::parse_lib_path),
            input: m.free,
        };
        args.check();
        args
    }

    fn check(&self) {
        if self.input.is_empty() && !self.cli && !self.version {
            eprintln!("error: required arguments were not provided: <INPUT>...");
            exit(2)
        }
    }

    fn parse_lib_path(path: impl AsRef<str>) -> Option<SmolStr> {
        path.as_ref()
            .parse()
            .map_err(|e| eprintln!("warning: {e}"))
            .ok()
    }

    fn parse_allow(lint: impl AsRef<str>) -> Option<lints::Lint> {
        lint.as_ref()
            .parse()
            .map_err(|e| eprintln!("warning: {e}"))
            .ok()
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

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut compiler = Compiler::new();

    if args.version {
        println!("OpenEX RustEdition v{}", compiler.get_version());
        println!("Copyright 2023-2026 by MCPPL,DotCS");
        return Ok(());
    }

    load_libraries(&mut compiler,args.lib,&args.allow)?;

    if args.cli {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                compiler.add_file(SourceFile::new("<console>".to_string(), input, args.allow));
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
            compiler.add_file(SourceFile::new(file_name, data, args.allow.clone()));
        }
    }

    compiler.compile();
    executor_run();
    Ok(())
}
