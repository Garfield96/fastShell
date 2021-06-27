use crate::commands::Command;
use crate::intermediate::Intermediate;
use crate::STANDALONE;
use rusqlite::{params, Connection};
use std::fs::File;
use std::io;
use std::io::BufRead;

pub struct cat;

impl Command for cat {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.transaction.push(
            "\
        DROP TABLE IF EXISTS data"
                .to_string(),
        );
        intermediate.transaction.push(
            "\
        CREATE TEMPORARY TABLE data (\
        lines text)"
                .to_string(),
        );
        intermediate.transaction.push(format!(
            "\
        COPY data FROM '{}'",
            parts[1]
        ));
        intermediate.sql = if STANDALONE {
            // let &mut mut conn = intermediate.conn.as_ref().unwrap().conn.as_mut().unwrap();
            // let file = File::open(parts[1]).unwrap();
            // let mut lines = io::BufReader::new(file).lines();
            // let mut t = conn.transaction().unwrap();
            // t.execute("CREATE TABLE data (lines TEXT NOT NULL)", [])
            //     .unwrap();
            // let mut stmt = t.prepare("INSERT INTO data (lines) VALUES (?1)").unwrap();
            // for line in lines {
            //     // println!("{}", line);
            //     stmt.execute(params![line.unwrap()]).unwrap();
            // }
            // stmt.finalize();
            // t.commit();
            format!("SELECT * FROM {}", "data")
        } else {
            format!("SELECT * FROM {}", parts[1])
        }
    }
}
