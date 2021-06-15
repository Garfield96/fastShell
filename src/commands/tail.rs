use crate::commands::Command;
use crate::Intermediate;

pub struct tail;

impl Command for tail {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = format!(
            "SELECT * FROM ({}) ORDER BY rowid desc LIMIT {}",
            intermediate.sql, parts[1]
        )
    }
}
