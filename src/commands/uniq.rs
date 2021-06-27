use crate::commands::Command;
use crate::intermediate::Intermediate;

pub struct uniq;

impl Command for uniq {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let flag = parts[1].replace("'", "");
        intermediate.sql = if parts.len() > 2 {
            if flag == "-c" || flag == "--count" {
                format!(
                    "SELECT count(*) || ' ' || {1} as lines FROM ({0}) as data GROUP BY {1}",
                    intermediate.sql,
                    parts[2].replace("'", "")
                )
            } else if flag == "-u" || flag == "--unique" {
                format!(
                    "SELECT * FROM ({0}) as data GROUP BY {1} HAVING count(*) = 1",
                    intermediate.sql,
                    parts[2].replace("'", "")
                )
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }
}
