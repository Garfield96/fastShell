use crate::commands::Command;
use crate::intermediate::Intermediate;

use rusqlite::functions::FunctionFlags;
use rusqlite::Connection;

use std::io::Write;
use std::process;
use std::process::Stdio;

pub struct shellCommand;

fn add_execexternal_function(db: &Connection) -> Result<(), rusqlite::Error> {
    db.create_scalar_function(
        "execexternal",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        move |ctx| {
            let prog_call: String = ctx.get(1).unwrap();
            let prog_call: Vec<&str> = prog_call.split(' ').collect();
            let input: String = ctx.get(0).unwrap();
            let mut prog = process::Command::new(prog_call.first().unwrap())
                .args(prog_call.iter().skip(1).cloned().collect::<Vec<&str>>())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdin = prog.stdin.take().unwrap();

            std::thread::spawn(move || {
                stdin.write_all(input.as_bytes()).unwrap();
            });

            let stdout = prog.wait_with_output().unwrap();
            let stdout = &*String::from_utf8_lossy(&stdout.stdout);
            let l = stdout.lines().next().unwrap();
            Ok(l.to_string())
        },
    );
    Ok(())
}

impl Command for shellCommand {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        // add_execexternal_function(&intermediate.conn.as_ref().unwrap().conn.unwrap());
        let cmd: String = parts
            .iter()
            .map(|s| String::from(s.as_str()))
            .collect::<Vec<String>>()
            .join(" ");
        intermediate.sql = format!(
            "SELECT execexternal(lines, '{}') as lines FROM ({})",
            cmd, intermediate.sql
        );
        // let mut prog = process::Command::new(parts[0])
        //     .args(parts.iter().skip(1).cloned().collect::<Vec<&String>>())
        //     .stdin(Stdio::piped())
        //     .stdout(Stdio::piped())
        //     .spawn()
        //     .unwrap();
        // let mut stdin = prog.stdin.take().unwrap();
        // let mut input = "".to_string();
        // if STANDALONE {
        //     let conn = intermediate.rs.conn.as_ref().unwrap();
        //     let stmt = conn.prepare(intermediate.rs.sql.as_str());
        //     let r_vec: Vec<String> = stmt
        //         .unwrap()
        //         .query_map([], |entry| entry.get::<_, String>(0))
        //         .unwrap()
        //         .into_iter()
        //         .map(|s| s.unwrap())
        //         .collect();
        //     input = r_vec.join("\n");
        // } else {
        //     let r = getxattr::getxattr(".", format!("sql#{}", intermediate.rs.sql), false).unwrap();
        //     input = String::from_utf8(r).unwrap();
        // }
        // std::thread::spawn(move || {
        //     stdin.write_all(input.as_bytes()).unwrap();
        // });
        // let stdout = prog.wait_with_output().unwrap();
        // let stdout = &*String::from_utf8_lossy(&stdout.stdout);
        //
        // let mut lines: Vec<&str> = stdout.lines().collect();
        // let mut conn: &mut Connection = intermediate.rs.conn.as_mut().unwrap();
        // let t = conn.transaction().unwrap();
        // t.execute("DROP TABLE data", []).unwrap();
        // t.execute("CREATE TABLE data (lines TEXT NOT NULL)", [])
        //     .unwrap();
        // let mut stmt = t.prepare("INSERT INTO data (lines) VALUES (?1)").unwrap();
        // for line in lines {
        //     // println!("{}", line);
        //     stmt.execute(params![line]).unwrap();
        // }
        // stmt.finalize();
        // t.commit();
        // intermediate.rs.sql = "SELECT * FROM data".to_string();
    }
}
