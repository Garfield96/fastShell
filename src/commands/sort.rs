use crate::commands::Command;
use crate::Intermediate;

pub struct sort;

impl Command for sort {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = if parts.len() > 2 {
            if parts[1] == "desc" {
                format!(
                    "SELECT * FROM ({}) ORDER BY {} desc",
                    intermediate.sql, parts[2]
                )
            } else {
                "".to_string()
            }
        } else {
            format!("SELECT * FROM ({}) ORDER BY {}", intermediate.sql, parts[1])
        };
    }
}
