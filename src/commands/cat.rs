use crate::commands::Command;
use crate::intermediate::Intermediate;
use crate::STANDALONE;
use clap::{App, Arg};

pub struct cat;

impl Command for cat {
    fn run(intermediate: &mut Intermediate, parts: Vec<&String>) {
        let matches = App::new("cat")
            .arg(Arg::with_name("file").required(true).takes_value(true))
            .get_matches_from(parts);
        if let Some(f) = matches.value_of("file") {
            if STANDALONE {
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
                intermediate
                    .transaction
                    .push("DROP TABLE IF EXISTS data".to_string());
                intermediate
                    .transaction
                    .push("CREATE TEMPORARY TABLE data (lines text)".to_string());
                intermediate
                    .transaction
                    .push(format!("COPY data FROM '{}'", f));
                intermediate.sql = format!("SELECT * FROM {}", "data")
            } else {
                intermediate.sql = format!("SELECT * FROM {}", f)
            }
        }
    }
}
