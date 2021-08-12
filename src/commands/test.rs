use crate::commands::Command;
use crate::intermediate::Intermediate;

#[allow(non_camel_case_types)]
pub struct test;

impl Command for test {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        for p in parts.iter().skip(1) {
            intermediate.sql.push_str(match p.as_str() {
                "-le" => "<=",
                "-lt" => "<",
                "-ge" => ">=",
                "-gt" => ">",
                "-eq" => "=",
                "-ne" => "!=",
                _ => p,
            });
            intermediate.sql.push(' ');
        }
    }
}
