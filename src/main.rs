use std::fs::File;
use std::io::Read;

use clap::{App, Arg};
// Build upon https://github.com/ipetkov/conch-parser/blob/master/examples/analysis.rs

use crate::executor::Executor;

mod commands;
mod db_backend;
mod executor;
mod getxattr;
mod intermediate;

const STANDALONE: bool = true;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = App::new("FastShell")
        .author("Christian Menges")
        .arg(
            Arg::with_name("COMMAND")
                .help("Execute command")
                .short("c")
                .value_name("command string")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("FILE")
                .help("Execute FILE")
                .short("f")
                .long("file")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();

    let script_file = args.value_of("FILE").unwrap_or("");
    let mut script;

    if !script_file.is_empty() {
        let mut input_file = File::open(script_file).expect("Cannot open input file");
        script = String::new();
        input_file.read_to_string(&mut script);
    } else {
        script = args.value_of("COMMAND").unwrap_or("").to_string();
    }

    let executor = Executor::create(&script);
    executor.run();

    Ok(())
}
