use crate::commands::Command;
use crate::Intermediate;

pub struct head;

impl Command for head {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = format!("SELECT * FROM ({}) LIMIT {}", intermediate.sql, parts[1]);
    }
}
