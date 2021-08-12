use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::{App, Arg};

#[allow(non_camel_case_types)]
pub struct uniq;

impl Command for uniq {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("uniq")
            .arg(
                Arg::with_name("count")
                    .short("c")
                    .value_name("column")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("unique")
                    .short("u")
                    .value_name("column")
                    .takes_value(true),
            )
            .get_matches_from(parts);

        if let Some(col) = matches.value_of("count") {
            intermediate.sql = format!(
                "SELECT count(*) || ' ' || {1} as lines FROM ({0}) as data GROUP BY {1}",
                intermediate.sql,
                col.replace("'", "")
            )
        }
        if let Some(col) = matches.value_of("unique") {
            intermediate.sql = format!(
                "SELECT * FROM ({0}) as data GROUP BY {1} HAVING count(*) = 1",
                intermediate.sql,
                col.replace("'", "")
            )
        }
    }
}
