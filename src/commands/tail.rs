use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::{App, Arg};

#[allow(non_camel_case_types)]
pub struct tail;

impl Command for tail {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("tail")
            .arg(
                Arg::with_name("line_count")
                    .short("n")
                    .required(true)
                    .takes_value(true),
            )
            .get_matches_from(parts);

        if let Some(n) = matches.value_of("line_count") {
            intermediate.sql = format!(
                "SELECT * FROM ({}) as data  ORDER BY rowid desc LIMIT {}",
                intermediate.sql, n
            )
        }
    }
}
