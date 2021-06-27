use crate::commands::Command;
use crate::intermediate::Intermediate;

pub struct shuf;

impl Command for shuf {
    fn run(intermediate: &mut Intermediate, _parts: Vec<&String>) {
        intermediate.sql = format!(
            "SELECT * FROM ({}) as data ORDER BY RANDOM()",
            intermediate.sql
        )
    }
}
