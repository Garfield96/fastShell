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
            // Based on https://stackoverflow.com/a/37980589
            intermediate.sql = format!(
                "SELECT lines FROM (\
                    SELECT * \
                    FROM (\
                        SELECT *, row_number() over (order by (select 1)) AS rowid FROM ({}) as data) as data \
                    ORDER BY rowid desc LIMIT {}) as data \
                ORDER BY rowid",
                intermediate.sql, n
            )
        }
    }
}
