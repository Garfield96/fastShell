use crate::commands::Command;
use crate::intermediate::Intermediate;

pub struct head;

impl Command for head {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = format!(
            "SELECT * FROM ({}) as data LIMIT {}",
            intermediate.sql,
            parts[1].replace("'", "")
        );
    }
}
