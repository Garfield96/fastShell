use crate::commands::Command;
use crate::{getxattr, Intermediate, STANDALONE};
use rusqlite::{params, Connection};
use std::borrow::Borrow;
use std::io::Write;
use std::process;
use std::process::Stdio;

pub struct shellCommand;

impl Command for shellCommand {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let mut prog = process::Command::new(parts[0])
            .args(parts.iter().skip(1).cloned().collect::<Vec<&String>>())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = prog.stdin.take().unwrap();
        let mut input = "".to_string();
        if STANDALONE {
            let conn = intermediate.conn.as_ref().unwrap();
            let stmt = conn.prepare(intermediate.sql.as_str());
            let r_vec: Vec<String> = stmt
                .unwrap()
                .query_map([], |entry| entry.get::<_, String>(0))
                .unwrap()
                .into_iter()
                .map(|s| s.unwrap())
                .collect();
            input = r_vec.join("\n");
        } else {
            let r = getxattr::getxattr(".", format!("sql#{}", intermediate.sql), false).unwrap();
            input = String::from_utf8(r).unwrap();
        }
        std::thread::spawn(move || {
            stdin.write_all(input.as_bytes()).unwrap();
        });
        let stdout = prog.wait_with_output().unwrap();
        let stdout = &*String::from_utf8_lossy(&stdout.stdout);

        let mut lines: Vec<&str> = stdout.lines().collect();
        let mut conn: &mut Connection = intermediate.conn.as_mut().unwrap();
        let t = conn.transaction().unwrap();
        t.execute("DROP TABLE data", []).unwrap();
        t.execute("CREATE TABLE data (lines TEXT NOT NULL)", [])
            .unwrap();
        let mut stmt = t.prepare("INSERT INTO data (lines) VALUES (?1)").unwrap();
        for line in lines {
            // println!("{}", line);
            stmt.execute(params![line]).unwrap();
        }
        stmt.finalize();
        t.commit();
        intermediate.sql = "SELECT * FROM data".to_string();
    }
}
