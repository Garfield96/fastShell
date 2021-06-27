use crate::commands::Command;
use crate::intermediate::Intermediate;

pub struct wc;

impl Command for wc {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let flag = parts[1].replace("'", "");
        intermediate.sql = if parts.len() > 1 {
            if flag == "-l" || flag == "--lines" {
                format!(
                    "SELECT cast(COUNT(*) as text) FROM ({}) as data ",
                    intermediate.sql
                )
            } else if flag == "-c" || flag == "--chars" {
                format!(
                    "SELECT cast((SUM(chars) + count(*)) as text) FROM (SELECT length(lines) as chars FROM ({}) as data) as data ",
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
