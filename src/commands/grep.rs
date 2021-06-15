use crate::commands::Command;
use crate::Intermediate;

pub struct grep;

impl Command for grep {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = format!(
            "SELECT * FROM ({}) WHERE {} GLOB '{}'",
            intermediate.sql, parts[1], parts[2]
        )
    }
}
