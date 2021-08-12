use postgres::Error;

use std::fmt;

// pub(crate) trait DB {
//     fn execute(&mut self, sql: String);
//     fn prepare<S>(&mut self, sql: String) -> &S;
// }

pub struct Postgres {
    pub conn: Option<postgres::Client>,
}

impl fmt::Debug for Postgres {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Postgres").finish()
    }
}

// impl DB for Postgres {
impl Postgres {
    pub fn execute(&mut self, sql: String) -> Result<u64, Error> {
        self.conn.as_mut().unwrap().execute(sql.as_str(), &[])
    }

    pub fn prepare(&mut self, sql: String) -> Result<postgres::Statement, Error> {
        self.conn.as_mut().unwrap().prepare(sql.as_str())
    }

    pub fn transaction(&mut self) -> Result<postgres::Transaction<'_>, Error> {
        self.conn.as_mut().unwrap().transaction()
    }

    pub fn batch_execute(&mut self, sql: String) -> Result<(), Error> {
        self.conn.as_mut().unwrap().batch_execute(&sql)
    }
}
