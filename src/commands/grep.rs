use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::{App, Arg};

#[allow(non_camel_case_types)]
pub struct grep;

impl Command for grep {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("grep")
            .arg(Arg::with_name("pattern").required(true).takes_value(true))
            .get_matches_from(parts);
        if let Some(p) = matches.value_of("pattern") {
            intermediate.sql = format!(
                "SELECT * FROM ({}) as data WHERE {} ~ '{}'",
                intermediate.sql, "lines", p
            )
        }
    }
}
