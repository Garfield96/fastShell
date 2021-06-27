use crate::db_backend::Postgres;
use conch_parser::ast::Redirect;

#[derive(Debug)]
pub struct Intermediate {
    pub(crate) transaction: Vec<String>,
    pub(crate) sql: String,
    pub(crate) conn: Option<Postgres>,
    // pub(crate) conn: Option<SQLite>,
    pub(crate) redirect: Option<Redirect<String>>,
}

impl Default for Intermediate {
    fn default() -> Self {
        Intermediate {
            transaction: Vec::new(),
            sql: "".to_string(),
            conn: None,
            redirect: None,
        }
    }
}

impl Intermediate {
    pub fn getSQL(&mut self) -> String {
        if !self.sql.is_empty() {
            self.transaction.push(format!(
                "COPY ({}) TO '/var/lib/postgresql/result.txt'",
                self.sql
            ));
            self.sql.clear();
        }
        let mut sql: String = String::new();
        if !self.transaction.is_empty() {
            sql = self.transaction.join("; ");
            self.transaction.clear();
            sql.push(';');
        }
        sql
    }
}
