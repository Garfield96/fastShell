use crate::commands::Command;
use crate::intermediate::Intermediate;

#[allow(non_camel_case_types)]
pub struct test;

impl Command for test {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        for p in parts.iter().skip(1) {
            intermediate.sql.push_str(&match p.as_str() {
                "-le" => "<=".to_string(),
                "-lt" => "<".to_string(),
                "-ge" => ">=".to_string(),
                "-gt" => ">".to_string(),
                "-eq" => "=".to_string(),
                "-ne" => "!=".to_string(),
                _ => {
                    if p.contains("SELECT") {
                        format!("({})", p)
                    } else {
                        format!("('{}')", p)
                    }
                }
            });
            intermediate.sql.push(' ');
        }
    }
}
