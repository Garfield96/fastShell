use crate::commands::Command;
use crate::intermediate::Intermediate;

#[allow(non_camel_case_types)]
pub struct shellCommand;

impl Command for shellCommand {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let cmd: String = parts
            .iter()
            .map(|s| String::from(s.as_str()))
            .collect::<Vec<String>>()
            .join(" ");

        intermediate.transaction.push(format!(
            "COPY ({}) TO PROGRAM '{} 1>/tmp/tmp' (FORMAT csv);\
            DELETE FROM data;
            COPY data FROM '/tmp/tmp' (FORMAT text);",
            intermediate.sql, cmd
        ));
        intermediate.sql = "SELECT * FROM data".to_string();
    }
}
