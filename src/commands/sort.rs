use crate::commands::Command;
use crate::intermediate::Intermediate;

pub struct sort;

impl Command for sort {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = if parts.len() > 2 {
            if parts[1] == "desc" {
                format!(
                    "SELECT * FROM ({}) as data  ORDER BY {} desc",
                    intermediate.sql,
                    parts[2].replace("'", "")
                )
            } else {
                "".to_string()
            }
        } else {
            format!(
                "SELECT * FROM ({}) as data  ORDER BY {}",
                intermediate.sql,
                parts[1].replace("'", "")
            )
        };
    }
}
