use crate::commands::Command;
use crate::intermediate::Intermediate;
use clap::App;

#[allow(non_camel_case_types)]
pub struct shuf;

impl Command for shuf {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let _matches = App::new("shuf").get_matches_from(parts);
        intermediate.sql = format!(
            "SELECT * FROM ({}) as data ORDER BY RANDOM()",
            intermediate.sql
        )
    }
}
