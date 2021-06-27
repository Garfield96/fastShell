use postgres::Error;
use rusqlite::{Connection, Statement, Transaction};
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::fmt::Display;

// pub(crate) trait DB {
//     fn execute(&mut self, sql: String);
//     fn prepare<S>(&mut self, sql: String) -> &S;
// }
#[derive(Debug)]
pub struct SQLite {
    pub conn: Option<rusqlite::Connection>,
}

// impl DB for SQLite {
impl SQLite {
    pub fn execute(&mut self, sql: String) {
        self.conn.as_ref().unwrap().execute(sql.as_str(), []);
    }

    pub fn prepare(&mut self, sql: String) -> Result<Statement<'_>, rusqlite::Error> {
        self.conn.as_ref().unwrap().prepare(sql.as_str())
    }

    pub fn transaction(&mut self) -> Result<Transaction<'_>, rusqlite::Error> {
        self.conn.as_mut().unwrap().transaction()
    }
}

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
    pub fn execute(&mut self, sql: String) {
        self.conn.as_mut().unwrap().execute(sql.as_str(), &[]);
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
