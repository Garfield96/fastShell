use crate::commands::Command;
use crate::Intermediate;

pub struct shuf;

impl Command for shuf {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = format!("SELECT * FROM ({}) ORDER BY RANDOM()", intermediate.sql)
    }
}
