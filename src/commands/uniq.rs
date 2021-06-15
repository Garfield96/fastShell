use crate::commands::Command;
use crate::Intermediate;

pub struct uniq;

impl Command for uniq {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = if parts.len() > 2 {
            if parts[1] == "-c" || parts[1] == "--count" {
                format!(
                    "SELECT count(*) || ' ' || {1} FROM ({0}) GROUP BY {1}",
                    intermediate.sql, parts[2]
                )
            } else if parts[1] == "-u" || parts[1] == "--unique" {
                format!(
                    "SELECT * FROM ({0}) GROUP BY {1} HAVING count(*) = 1",
                    intermediate.sql, parts[2]
                )
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }
}
