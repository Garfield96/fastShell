use crate::commands::Command;
use crate::intermediate::Intermediate;

#[allow(non_camel_case_types)]
pub struct echo;

impl Command for echo {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = format!(
            "SELECT ('{}')",
            parts[1].replace("\"", "").replace("\'", "")
        );
    }
}
