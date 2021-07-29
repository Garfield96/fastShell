use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::{App, Arg};

pub struct wc;

impl Command for wc {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("wc")
            .arg(
                Arg::with_name("lines")
                    .short("l")
                    .required_unless("chars")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("chars")
                    .short("c")
                    .required_unless("lines")
                    .takes_value(false),
            )
            .get_matches_from(parts);

        if matches.is_present("lines") {
            intermediate.sql = format!(
                "SELECT cast(COUNT(*) as text) FROM ({}) as data ",
                intermediate.sql
            )
        }
        if matches.is_present("chars") {
            intermediate.sql = format!(
                "SELECT cast((SUM(chars) + count(*)) as text) FROM (SELECT length(lines) as chars FROM ({}) as data) as data ",
                intermediate.sql
            )
        }
    }
}
