use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::{App, Arg};

pub struct sort;

impl Command for sort {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("sort")
            .arg(Arg::with_name("reverse").short("r").takes_value(false))
            .arg(
                Arg::with_name("ignore-leading-blanks")
                    .short("b")
                    .takes_value(false),
            )
            .arg(Arg::with_name("ignore-case").short("f").takes_value(false))
            .get_matches_from(parts);

        let order = if matches.is_present("reverse") {
            "desc"
        } else {
            ""
        };
        let mut comp = "lines".to_string();
        comp = if matches.is_present("ignore-leading-blanks") {
            format!("TRIM({})", comp)
        } else {
            comp
        };
        comp = if matches.is_present("ignore-case") {
            format!("UPPER({})", comp)
        } else {
            comp
        };
        intermediate.sql = format!(
            "SELECT * FROM ({}) as data  ORDER BY {} {}",
            intermediate.sql, comp, order
        );
    }
}
