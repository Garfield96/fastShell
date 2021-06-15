use crate::commands::Command;
use crate::{Intermediate, STANDALONE};
use rusqlite::{params, Connection};
use std::fs::File;
use std::io;
use std::io::BufRead;

pub struct cat;

impl Command for cat {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        intermediate.sql = if STANDALONE {
            let mut conn = Connection::open_in_memory().unwrap();
            let file = File::open(parts[1]).unwrap();
            let mut lines = io::BufReader::new(file).lines();
            let t = conn.transaction().unwrap();
            t.execute("CREATE TABLE data (lines TEXT NOT NULL)", [])
                .unwrap();
            let mut stmt = t.prepare("INSERT INTO data (lines) VALUES (?1)").unwrap();
            for line in lines {
                // println!("{}", line);
                stmt.execute(params![line.unwrap()]).unwrap();
            }
            stmt.finalize();
            t.commit();
            intermediate.conn = Some(conn);
            format!("SELECT * FROM {}", "data")
        } else {
            format!("SELECT * FROM {}", parts[1])
        }
    }
}
