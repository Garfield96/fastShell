use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::{App, Arg};

pub struct head;

impl Command for head {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("head")
            .arg(
                Arg::with_name("line_count")
                    .short("n")
                    .required(true)
                    .takes_value(true),
            )
            .get_matches_from(parts);

        if let Some(n) = matches.value_of("line_count") {
            intermediate.sql = format!(
                "SELECT * FROM ({}) as data LIMIT {}",
                intermediate.sql,
                n.replace("'", "")
            );
        }
    }
}
