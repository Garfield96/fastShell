use crate::commands::Command;
use crate::Intermediate;

pub struct wc;

impl Command for wc {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = if parts.len() > 1 {
            if parts[1] == "-l" || parts[1] == "--lines" {
                format!(
                    "SELECT cast(COUNT(lines) as text) FROM ({})",
                    intermediate.sql
                )
            } else if parts[1] == "-c" || parts[1] == "--chars" {
                format!(
                    "SELECT cast((SUM(chars) + count(*) )as text) FROM (SELECT length(lines) as chars FROM ({}))",
                    intermediate.sql
                )
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }
}
